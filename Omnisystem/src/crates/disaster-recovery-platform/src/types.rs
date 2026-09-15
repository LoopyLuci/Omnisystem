//! Data types
use serde::{Deserialize, Serialize};

/// A recovery plan with its RPO/RTO targets and ordered execution steps.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryPlan {
    /// Plan name, unique within the platform.
    pub name: String,
    /// Recovery Point Objective: maximum acceptable data loss, in ticks
    /// since the most recent snapshot.
    pub rpo_ticks: u64,
    /// Recovery Time Objective: maximum acceptable time to complete
    /// recovery, in ticks since the disaster was declared.
    pub rto_ticks: u64,
    /// Ordered list of step names to execute during a drill or real recovery.
    pub steps: Vec<String>,
}

/// Outcome of a completed drill (or real recovery) against a plan.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DrillResult {
    /// The plan that was executed.
    pub plan_name: String,
    /// Tick at which the disaster was declared / drill started.
    pub disaster_tick: u64,
    /// Tick at which all steps completed.
    pub completed_tick: u64,
    /// Actual recovery time observed (`completed_tick - disaster_tick`).
    pub actual_rto_ticks: u64,
    /// Whether the actual recovery time was within the plan's RTO target.
    pub rto_met: bool,
    /// Data-loss window observed: `disaster_tick - most_recent_snapshot_tick`.
    /// `None` if no snapshot existed before the disaster.
    pub actual_rpo_ticks: Option<u64>,
    /// Whether the observed data-loss window was within the plan's RPO target.
    pub rpo_met: bool,
    /// Number of steps completed (should equal the plan's step count).
    pub steps_completed: usize,
    /// Total number of steps in the plan.
    pub steps_total: usize,
}

/// A single in-progress drill/recovery execution.
#[derive(Debug, Clone)]
pub(crate) struct Execution {
    pub started_tick: u64,
    pub next_step: usize,
}
