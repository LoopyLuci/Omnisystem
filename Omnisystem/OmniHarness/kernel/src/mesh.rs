use anyhow::Result;
use dashmap::DashMap;
use futures::StreamExt;   // provides Swarm::select_next_some
use libp2p::{
    gossipsub::{self, IdentTopic as Topic, MessageAuthenticity},
    identify, mdns,
    swarm::{NetworkBehaviour, SwarmEvent},
    PeerId, SwarmBuilder,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

/// peer_id -> (multiaddr, discovered_at_ms). Shared between the mesh task
/// (which owns the swarm and is the only thing that can see live peers) and
/// `MeshHandle` (cloned into gRPC handlers), so `MeshService::ListPeers` can
/// report real, currently-known peers without needing swarm access itself.
type PeerTable = Arc<DashMap<String, (String, i64)>>;

/// Handle other parts of the kernel (gRPC handlers) use to talk to the mesh
/// task without owning the `Swarm` themselves — the swarm only lives inside
/// `MeshNode::run`'s task, so outbound broadcasts go through a channel and
/// peer info is read from the shared `PeerTable` the run loop maintains.
#[derive(Clone)]
pub struct MeshHandle {
    broadcast_tx: mpsc::Sender<Vec<u8>>,
    peers: PeerTable,
}

impl MeshHandle {
    /// Publish `data` to the gossipsub topic. Returns once the mesh task has
    /// accepted the payload for publishing (fire-and-forget past that point,
    /// same as libp2p gossipsub itself).
    pub async fn broadcast(&self, data: Vec<u8>) -> Result<()> {
        self.broadcast_tx
            .send(data)
            .await
            .map_err(|_| anyhow::anyhow!("mesh task is not running"))
    }

    pub fn peer_count(&self) -> usize {
        self.peers.len()
    }

    pub fn list_peers(&self) -> Vec<(String, String, i64)> {
        self.peers
            .iter()
            .map(|e| (e.key().clone(), e.value().0.clone(), e.value().1))
            .collect()
    }
}

/// Build a `MeshNode`, wire it up, and spawn its run loop. Returns the task
/// handle (so callers can `abort()` it on shutdown) plus a `MeshHandle` for
/// issuing broadcasts / reading peers from gRPC handlers.
pub fn spawn(event_tx: mpsc::Sender<Vec<u8>>) -> Result<(tokio::task::JoinHandle<()>, MeshHandle)> {
    let node = MeshNode::new(event_tx)?;
    let (broadcast_tx, broadcast_rx) = mpsc::channel::<Vec<u8>>(256);
    let peers: PeerTable = Arc::new(DashMap::new());
    let handle = MeshHandle { broadcast_tx, peers: Arc::clone(&peers) };
    let join = tokio::spawn(async move { node.run(broadcast_rx, peers).await });
    Ok((join, handle))
}

#[derive(NetworkBehaviour)]
pub struct HarnessBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub mdns:      mdns::tokio::Behaviour,
    pub identify:  identify::Behaviour,
}

pub struct MeshNode {
    swarm:        libp2p::Swarm<HarnessBehaviour>,
    event_sender: mpsc::Sender<Vec<u8>>,
    topic:        Topic,
}

impl MeshNode {
    pub fn new(tx: mpsc::Sender<Vec<u8>>) -> Result<Self> {
        let key     = libp2p::identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from(key.public());
        info!("[Mesh] Local peer ID: {}", peer_id);

        let gossipsub_cfg = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(1))
            .validation_mode(gossipsub::ValidationMode::Strict)
            .build()
            .map_err(|e| anyhow::anyhow!("gossipsub config: {}", e))?;

        let gossipsub = gossipsub::Behaviour::new(
            MessageAuthenticity::Signed(key.clone()),
            gossipsub_cfg,
        ).map_err(|e| anyhow::anyhow!("gossipsub: {}", e))?;

        let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), peer_id)
            .map_err(|e| anyhow::anyhow!("mdns: {}", e))?;

        let identify = identify::Behaviour::new(identify::Config::new(
            "/omniharness/1.0.0".to_string(),
            key.public(),
        ));

        let behaviour = HarnessBehaviour { gossipsub, mdns, identify };

        let swarm = SwarmBuilder::with_existing_identity(key)
            .with_tokio()
            .with_tcp(
                libp2p::tcp::Config::default(),
                libp2p::noise::Config::new,
                libp2p::yamux::Config::default,
            )?
            .with_behaviour(|_| behaviour)?
            .build();

        let topic = Topic::new("omniharness-events");

        Ok(Self { swarm, event_sender: tx, topic })
    }

    pub fn broadcast(&mut self, data: Vec<u8>) -> Result<()> {
        self.swarm
            .behaviour_mut()
            .gossipsub
            .publish(self.topic.clone(), data)
            .map(|_| ())
            .map_err(|e| anyhow::anyhow!("publish: {:?}", e))
    }

    pub fn peer_count(&self) -> usize {
        self.swarm.behaviour().gossipsub.all_peers().count()
    }

    /// Drive the swarm event loop AND service outbound broadcast requests
    /// coming from `MeshHandle::broadcast` (i.e. from `MeshService::BroadcastEvent`
    /// gRPC calls) — the swarm can only be driven from inside this task, so
    /// outbound publishes have to come in over `broadcast_rx` rather than a
    /// direct method call from the gRPC handler's task.
    pub async fn run(mut self, mut broadcast_rx: mpsc::Receiver<Vec<u8>>, peers: PeerTable) {
        if let Err(e) = self.swarm.behaviour_mut().gossipsub.subscribe(&self.topic) {
            error!("[Mesh] Subscribe failed: {}", e);
            return;
        }
        if let Err(e) = self.swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse().unwrap()) {
            error!("[Mesh] Listen failed: {}", e);
            return;
        }

        loop {
            tokio::select! {
                outbound = broadcast_rx.recv() => {
                    match outbound {
                        Some(data) => {
                            if let Err(e) = self.broadcast(data) {
                                warn!("[Mesh] Broadcast failed: {}", e);
                            }
                        }
                        None => {
                            warn!("[Mesh] Broadcast channel closed — outbound publishing disabled.");
                            // Keep servicing inbound swarm events even if every
                            // MeshHandle has been dropped.
                        }
                    }
                }
                event = self.swarm.select_next_some() => {
                    match event {
                        SwarmEvent::Behaviour(HarnessBehaviourEvent::Gossipsub(
                            gossipsub::Event::Message { message, .. }
                        )) => {
                            if self.event_sender.send(message.data).await.is_err() {
                                warn!("[Mesh] Channel closed — shutting down mesh.");
                                return;
                            }
                        }
                        SwarmEvent::Behaviour(HarnessBehaviourEvent::Mdns(
                            mdns::Event::Discovered(list)
                        )) => {
                            for (peer_id, addr) in list {
                                info!("[Mesh] Discovered peer {} at {}", peer_id, addr);
                                peers.insert(peer_id.to_string(), (addr.to_string(), chrono::Utc::now().timestamp_millis()));
                                self.swarm.dial(peer_id).ok();
                            }
                        }
                        SwarmEvent::Behaviour(HarnessBehaviourEvent::Mdns(
                            mdns::Event::Expired(list)
                        )) => {
                            for (peer_id, _) in list {
                                info!("[Mesh] Peer {} expired.", peer_id);
                                peers.remove(&peer_id.to_string());
                            }
                        }
                        SwarmEvent::NewListenAddr { address, .. } => {
                            info!("[Mesh] Listening on {}", address);
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}
