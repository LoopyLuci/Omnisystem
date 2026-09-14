/// Health Monitoring
/// Continuous health monitoring of relay nodes

use crate::node::RelayNode;
use dashmap::DashMap;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, warn};

pub struct HealthMonitor;

impl HealthMonitor {
    pub fn new() -> Self {
        HealthMonitor
    }

    pub async fn start_monitoring(
        &self,
        nodes: Arc<DashMap<String, RelayNode>>,
    ) {
        tokio::spawn(async move {
            loop {
                debug!("Running health check on {} nodes", nodes.len());

                // Check each node's health
                for mut entry in nodes.iter_mut() {
                    let node = entry.value_mut();
                    let is_healthy = node.is_healthy(300);

                    if !is_healthy {
                        warn!("Node {} is unhealthy", node.info.id);
                        // Decrease reliability
                        node.info.reliability *= 0.95;
                    }
                }

                tokio::time::sleep(Duration::from_secs(60)).await;
            }
        });
    }

    pub fn calculate_network_health(nodes: &[RelayNode]) -> f64 {
        if nodes.is_empty() {
            return 0.0;
        }

        let healthy_count = nodes.iter().filter(|n| n.is_healthy(300)).count();
        healthy_count as f64 / nodes.len() as f64
    }
}

impl Default for HealthMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_monitor_creation() {
        let _monitor = HealthMonitor::new();
    }

    #[test]
    fn test_network_health_calculation() {
        let nodes = vec![];
        let health = HealthMonitor::calculate_network_health(&nodes);
        assert_eq!(health, 0.0);
    }
}
