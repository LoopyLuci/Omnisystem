use serde::{Deserialize, Serialize};

/// Per-region bookkeeping: the tick at which it last sent a heartbeat.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct RegionState {
    /// Logical tick of the region's most recent heartbeat.
    pub last_heartbeat_tick: u64,
}

/// Snapshot of global failover status at a given tick.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FailoverStatus {
    /// The region currently serving as active primary, if any region is alive.
    pub active_primary: Option<String>,
    /// The region designated as primary at cluster configuration time.
    pub original_primary: String,
    /// True once the active primary has failed over away from the original.
    pub failed_over: bool,
    /// Total number of regions failovers performed so far (monotonic counter).
    pub failover_count: u32,
    /// Number of regions currently considered alive.
    pub alive_regions: usize,
    /// Total number of regions registered in the failover group.
    pub total_regions: usize,
}
