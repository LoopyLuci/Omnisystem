use serde::{Deserialize, Serialize};

/// Aggregate health of a single check.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// How many consecutive failures/successes are required to flip status.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct CheckConfig {
    pub failure_threshold: u32,
    pub success_threshold: u32,
}

impl CheckConfig {
    pub fn new(failure_threshold: u32, success_threshold: u32) -> Self {
        Self {
            failure_threshold: failure_threshold.max(1),
            success_threshold: success_threshold.max(1),
        }
    }
}

impl Default for CheckConfig {
    fn default() -> Self {
        Self::new(3, 2)
    }
}

/// Running state for one registered check.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CheckState {
    pub config: CheckConfig,
    pub status: HealthStatus,
    pub consecutive_failures: u32,
    pub consecutive_successes: u32,
}

impl CheckState {
    pub fn new(config: CheckConfig) -> Self {
        Self {
            config,
            status: HealthStatus::Healthy,
            consecutive_failures: 0,
            consecutive_successes: 0,
        }
    }
}
