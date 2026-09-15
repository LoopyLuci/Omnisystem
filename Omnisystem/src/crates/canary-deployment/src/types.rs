use serde::{Deserialize, Serialize};

/// Outcome of feeding a new error-rate sample into the controller.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum CanaryDecision {
    /// Error rate was acceptable; rollout advanced to this traffic percent.
    Advanced(u8),
    /// Error rate exceeded the threshold; rollout was aborted and traffic
    /// reverted to 0% canary.
    RolledBack,
    /// The final stage's error rate was acceptable; the canary now serves
    /// 100% of traffic and the rollout is finished.
    Completed,
}

/// A single recorded observation during the rollout.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StageObservation {
    pub stage_percent: u8,
    pub error_rate: f64,
    pub decision: CanaryDecision,
}
