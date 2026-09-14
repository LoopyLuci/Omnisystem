/// Peer Discovery
/// DHT-based peer discovery for relay network

use std::collections::HashSet;
use tracing::{debug, info};

pub struct PeerDiscovery {
    known_peers: HashSet<String>,
    bootstrap_nodes: Vec<String>,
}

impl PeerDiscovery {
    pub fn new() -> Self {
        let bootstrap_nodes = vec![
            "seed1.aether-relay.io".to_string(),
            "seed2.aether-relay.io".to_string(),
            "seed3.aether-relay.io".to_string(),
        ];

        PeerDiscovery {
            known_peers: HashSet::new(),
            bootstrap_nodes,
        }
    }

    pub async fn discover_peers(&mut self) -> anyhow::Result<Vec<String>> {
        let mut peers = Vec::new();

        debug!("Starting peer discovery with {} bootstrap nodes", self.bootstrap_nodes.len());

        for bootstrap in &self.bootstrap_nodes {
            match self.query_bootstrap_node(bootstrap).await {
                Ok(node_peers) => {
                    debug!("Bootstrap {} returned {} peers", bootstrap, node_peers.len());
                    peers.extend(node_peers);
                }
                Err(e) => {
                    debug!("Bootstrap {} query failed: {}", bootstrap, e);
                    // Continue with next bootstrap node
                }
            }
        }

        // Add bootstrap nodes themselves
        peers.extend(self.bootstrap_nodes.clone());

        // Remove duplicates
        peers.sort();
        peers.dedup();

        info!("Peer discovery complete: {} peers found", peers.len());

        // Update known peers
        for peer in &peers {
            self.known_peers.insert(peer.clone());
        }

        Ok(peers)
    }

    async fn query_bootstrap_node(&self, bootstrap: &str) -> anyhow::Result<Vec<String>> {
        debug!("Querying bootstrap node: {}", bootstrap);

        // In production, would make actual network request
        // For now, return empty list (demonstrates integration)
        // Real implementation would use DHT protocol or custom RPC

        Ok(vec![])
    }

    pub fn add_peer(&mut self, peer: String) {
        debug!("Adding peer: {}", peer);
        self.known_peers.insert(peer);
    }

    pub fn add_peers(&mut self, peers: Vec<String>) {
        for peer in peers {
            self.add_peer(peer);
        }
    }

    pub fn peer_count(&self) -> usize {
        self.known_peers.len()
    }

    pub fn get_peers(&self) -> Vec<String> {
        let mut peers: Vec<_> = self.known_peers.iter().cloned().collect();
        peers.sort();
        peers
    }

    pub fn get_random_peer(&self) -> Option<String> {
        use rand::seq::SliceRandom;
        let peers: Vec<_> = self.known_peers.iter().cloned().collect();
        if peers.is_empty() {
            None
        } else {
            let mut rng = rand::thread_rng();
            peers.choose(&mut rng).cloned()
        }
    }

    pub fn set_bootstrap_nodes(&mut self, nodes: Vec<String>) {
        self.bootstrap_nodes = nodes;
    }
}

impl Default for PeerDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peer_discovery_creation() {
        let discovery = PeerDiscovery::new();
        assert_eq!(discovery.bootstrap_nodes.len(), 3);
        assert_eq!(discovery.peer_count(), 0);
    }

    #[test]
    fn test_add_peer() {
        let mut discovery = PeerDiscovery::new();
        discovery.add_peer("peer1.example.com".to_string());
        assert_eq!(discovery.peer_count(), 1);
    }

    #[test]
    fn test_add_peers() {
        let mut discovery = PeerDiscovery::new();
        let peers = vec![
            "peer1.example.com".to_string(),
            "peer2.example.com".to_string(),
        ];
        discovery.add_peers(peers);
        assert_eq!(discovery.peer_count(), 2);
    }

    #[tokio::test]
    async fn test_discover_peers() {
        let mut discovery = PeerDiscovery::new();
        let result = discovery.discover_peers().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_random_peer() {
        let mut discovery = PeerDiscovery::new();
        discovery.add_peer("peer1.example.com".to_string());
        let peer = discovery.get_random_peer();
        assert!(peer.is_some());
    }
}
