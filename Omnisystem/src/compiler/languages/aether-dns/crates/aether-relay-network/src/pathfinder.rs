/// Path Finding
/// Optimal relay path selection

use crate::node::RelayNode;
use dashmap::DashMap;
use std::collections::HashSet;

pub struct PathFinder;

impl PathFinder {
    pub fn new() -> Self {
        PathFinder
    }

    pub fn find_optimal_path(
        &self,
        nodes: &DashMap<String, RelayNode>,
        hop_count: usize,
    ) -> anyhow::Result<Vec<RelayNode>> {
        let mut path = Vec::new();
        let mut used_asns = HashSet::new();
        let mut used_countries = HashSet::new();

        // Collect all healthy nodes
        let mut candidates: Vec<_> = nodes
            .iter()
            .filter(|n| n.is_healthy(300))
            .map(|n| n.value().clone())
            .collect();

        // Sort by score (highest first)
        candidates.sort_by(|a, b| b.score().partial_cmp(&a.score()).unwrap());

        // Select diverse nodes
        for node in &candidates {
            if path.len() >= hop_count {
                break;
            }

            // Prefer different ASNs and countries
            if used_asns.contains(&node.info.location.asn)
                || used_countries.contains(&node.info.location.country)
            {
                continue;
            }

            path.push(node.clone());
            used_asns.insert(node.info.location.asn);
            used_countries.insert(node.info.location.country.clone());
        }

        if path.len() < hop_count {
            // Fall back to any healthy nodes if diversity not possible
            for node in &candidates {
                if path.len() >= hop_count {
                    break;
                }
                if !path.iter().any(|n| n.info.id == node.info.id) {
                    path.push(node.clone());
                }
            }
        }

        if path.len() < hop_count {
            return Err(anyhow::anyhow!(
                "Not enough nodes for path: requested {}, found {}",
                hop_count,
                path.len()
            ));
        }

        Ok(path)
    }

    pub fn calculate_path_latency(path: &[RelayNode]) -> u32 {
        path.iter().map(|n| n.info.latency_ms).sum()
    }

    pub fn calculate_path_reliability(path: &[RelayNode]) -> f64 {
        path.iter()
            .map(|n| n.info.reliability)
            .product()
    }
}

impl Default for PathFinder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pathfinder_creation() {
        let _pf = PathFinder::new();
    }

    #[test]
    fn test_latency_calculation() {
        let nodes = vec![];
        let latency = PathFinder::calculate_path_latency(&nodes);
        assert_eq!(latency, 0);
    }
}
