use crate::error::{Error, Result};
use crate::types::{ClusterStatus, NodeState};
use std::collections::HashMap;
use std::sync::RwLock;

/// Cluster membership, quorum computation, and deterministic leader
/// election, driven by a caller-supplied logical `tick` rather than wall
/// clock time (so behaviour is fully reproducible in tests). A node is
/// considered alive if it has heartbeated within `heartbeat_timeout_ticks`
/// of the tick passed to `evaluate`. Quorum requires a strict majority of
/// all known nodes to be alive; the leader is the lexicographically
/// smallest alive node id, re-elected whenever the current leader is no
/// longer alive.
pub struct Manager {
    nodes: RwLock<HashMap<String, NodeState>>,
    heartbeat_timeout_ticks: u64,
    leader: RwLock<Option<String>>,
}

impl Manager {
    pub fn new(heartbeat_timeout_ticks: u64) -> Self {
        Self {
            nodes: RwLock::new(HashMap::new()),
            heartbeat_timeout_ticks,
            leader: RwLock::new(None),
        }
    }

    pub fn join(&self, node_id: &str, tick: u64) {
        self.nodes
            .write()
            .unwrap()
            .insert(node_id.to_string(), NodeState { last_heartbeat_tick: tick });
    }

    pub fn heartbeat(&self, node_id: &str, tick: u64) -> Result<()> {
        let mut nodes = self.nodes.write().unwrap();
        let node = nodes
            .get_mut(node_id)
            .ok_or_else(|| Error::UnknownNode(node_id.to_string()))?;
        node.last_heartbeat_tick = tick;
        Ok(())
    }

    fn alive_nodes(&self, tick: u64) -> Vec<String> {
        let nodes = self.nodes.read().unwrap();
        let mut alive: Vec<String> = nodes
            .iter()
            .filter(|(_, state)| tick.saturating_sub(state.last_heartbeat_tick) <= self.heartbeat_timeout_ticks)
            .map(|(id, _)| id.clone())
            .collect();
        alive.sort();
        alive
    }

    /// Recompute quorum and leadership as of `tick`.
    pub fn evaluate(&self, tick: u64) -> ClusterStatus {
        let total = self.nodes.read().unwrap().len();
        let alive = self.alive_nodes(tick);
        let has_quorum = total > 0 && alive.len() * 2 > total;

        let mut leader = self.leader.write().unwrap();
        let leader_still_alive = leader.as_ref().is_some_and(|l| alive.contains(l));

        if !has_quorum {
            *leader = None;
        } else if !leader_still_alive {
            *leader = alive.first().cloned();
        }

        ClusterStatus {
            total_nodes: total,
            alive_nodes: alive.len(),
            has_quorum,
            leader: leader.clone(),
        }
    }

    pub fn current_leader(&self) -> Option<String> {
        self.leader.read().unwrap().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cluster_of_three() -> Manager {
        let m = Manager::new(5);
        m.join("a", 0);
        m.join("b", 0);
        m.join("c", 0);
        m
    }

    #[test]
    fn quorum_and_leader_when_all_alive() {
        let m = cluster_of_three();
        let status = m.evaluate(1);
        assert!(status.has_quorum);
        assert_eq!(status.alive_nodes, 3);
        assert_eq!(status.leader.as_deref(), Some("a"));
    }

    #[test]
    fn quorum_lost_when_majority_goes_stale() {
        let m = cluster_of_three();
        m.heartbeat("a", 0).unwrap();
        // b and c never heartbeat again; jump tick far past timeout.
        let status = m.evaluate(100);
        assert!(!status.has_quorum);
        assert_eq!(status.leader, None);
    }

    #[test]
    fn quorum_holds_with_bare_majority() {
        let m = cluster_of_three();
        m.heartbeat("a", 10).unwrap();
        m.heartbeat("b", 10).unwrap();
        // c stops heartbeating after tick 0, so by tick 12 only c is stale.
        let status = m.evaluate(12);
        assert!(status.has_quorum);
        assert_eq!(status.alive_nodes, 2);
    }

    #[test]
    fn leader_failover_when_leader_goes_stale() {
        let m = cluster_of_three();
        let first = m.evaluate(0);
        assert_eq!(first.leader.as_deref(), Some("a"));

        // Only b and c keep heartbeating; a goes stale by tick 12.
        m.heartbeat("b", 10).unwrap();
        m.heartbeat("c", 10).unwrap();
        let status = m.evaluate(12);
        assert!(status.has_quorum);
        assert_eq!(status.leader.as_deref(), Some("b"));
    }

    #[test]
    fn heartbeat_on_unknown_node_errors() {
        let m = Manager::new(5);
        assert!(matches!(m.heartbeat("ghost", 0).unwrap_err(), Error::UnknownNode(_)));
    }

    #[test]
    fn empty_cluster_has_no_quorum() {
        let m = Manager::new(5);
        let status = m.evaluate(0);
        assert!(!status.has_quorum);
        assert_eq!(status.leader, None);
    }
}
