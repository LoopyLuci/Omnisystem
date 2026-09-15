use serde::{Deserialize, Serialize};

/// Per-node bookkeeping: the tick at which it last sent a heartbeat.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct NodeState {
    pub last_heartbeat_tick: u64,
}

/// Snapshot of cluster health at a given tick.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ClusterStatus {
    pub total_nodes: usize,
    pub alive_nodes: usize,
    pub has_quorum: bool,
    pub leader: Option<String>,
}
