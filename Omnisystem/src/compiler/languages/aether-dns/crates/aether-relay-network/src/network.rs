/// Relay Network Management

use crate::node::RelayNode;
use crate::health::HealthMonitor;
use crate::pathfinder::PathFinder;
use crate::discovery::PeerDiscovery;
use dashmap::DashMap;
use std::sync::Arc;
use tracing::info;

pub struct RelayNetwork {
    nodes: Arc<DashMap<String, RelayNode>>,
    health_monitor: Arc<HealthMonitor>,
    pathfinder: Arc<PathFinder>,
    peer_discovery: Arc<tokio::sync::Mutex<PeerDiscovery>>,
}

impl RelayNetwork {
    pub fn new() -> Self {
        RelayNetwork {
            nodes: Arc::new(DashMap::new()),
            health_monitor: Arc::new(HealthMonitor::new()),
            pathfinder: Arc::new(PathFinder::new()),
            peer_discovery: Arc::new(tokio::sync::Mutex::new(PeerDiscovery::new())),
        }
    }

    pub async fn initialize(&self) -> anyhow::Result<()> {
        info!("Initializing relay network");

        // Start peer discovery
        let mut discovery = self.peer_discovery.lock().await;
        let peers = discovery.discover_peers().await?;

        info!("Discovered {} relay peers", peers.len());

        // In production, would load actual relay nodes and register them
        // For now, just mark as initialized

        Ok(())
    }

    pub fn register_node(&self, node: RelayNode) {
        let node_id = node.info.id.clone();
        self.nodes.insert(node_id.clone(), node);
        info!("Registered relay node: {}", node_id);
    }

    pub fn get_node(&self, node_id: &str) -> Option<RelayNode> {
        self.nodes.get(node_id).map(|n| n.clone())
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub async fn select_optimal_path(
        &self,
        hop_count: usize,
    ) -> anyhow::Result<Vec<RelayNode>> {
        if self.nodes.len() < hop_count {
            return Err(anyhow::anyhow!("Not enough nodes for path"));
        }

        self.pathfinder.find_optimal_path(&self.nodes, hop_count)
    }

    pub async fn monitor_health(&self) {
        self.health_monitor.start_monitoring(Arc::clone(&self.nodes))
            .await;
    }

    pub fn healthy_nodes(&self) -> Vec<RelayNode> {
        self.nodes
            .iter()
            .filter(|n| n.is_healthy(300))
            .map(|n| n.clone())
            .collect()
    }

    pub fn network_health(&self) -> f64 {
        let total = self.nodes.len();
        if total == 0 {
            return 0.0;
        }

        let healthy = self.healthy_nodes().len();
        healthy as f64 / total as f64
    }

    pub async fn get_random_relay(&self) -> Option<RelayNode> {
        let healthy = self.healthy_nodes();
        if healthy.is_empty() {
            None
        } else {
            use rand::seq::SliceRandom;
            let mut rng = rand::thread_rng();
            healthy.choose(&mut rng).cloned()
        }
    }
}

impl Default for RelayNetwork {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_creation() {
        let network = RelayNetwork::new();
        assert_eq!(network.node_count(), 0);
    }

    #[tokio::test]
    async fn test_node_registration() {
        let network = RelayNetwork::new();

        let info = crate::node::RelayNodeInfo {
            id: "node1".to_string(),
            pubkey: vec![0; 32],
            endpoints: vec!["127.0.0.1:9000".to_string()],
            latency_ms: 50,
            bandwidth_mbps: 1000,
            reliability: 0.99,
            privacy_rating: 4.8,
            location: crate::node::GeoLocation {
                country: "US".to_string(),
                city: "SF".to_string(),
                asn: 1234,
                latitude: 37.7749,
                longitude: -122.4194,
            },
            supports_ipv6: true,
            supports_quic: true,
            supports_tls: true,
        };

        let node = RelayNode::new(info);
        network.register_node(node);
        assert_eq!(network.node_count(), 1);
    }

    #[tokio::test]
    async fn test_initialize() {
        let network = RelayNetwork::new();
        let result = network.initialize().await;
        assert!(result.is_ok());
    }
}
