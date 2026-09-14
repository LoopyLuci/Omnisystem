/// Worker Trait - Universal Task Execution Interface

use async_trait::async_trait;
use std::time::Duration;
use serde::{Deserialize, Serialize};

pub type WorkerResult<T> = Result<T, crate::WorkerError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Priority {
    Critical = 100,
    High = 75,
    Normal = 50,
    Low = 25,
    Background = 0,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Recovering,
}

/// Universal worker trait for all task types
#[async_trait]
pub trait Worker: Send + Sync + 'static {
    type Input: Send + 'static;
    type Output: Send + 'static;

    /// Execute the worker task
    async fn execute(&self, input: Self::Input) -> WorkerResult<Self::Output>;

    /// Get worker priority
    fn priority(&self) -> Priority {
        Priority::Normal
    }

    /// Maximum execution time
    fn timeout(&self) -> Duration {
        Duration::from_secs(300)
    }

    /// Maximum retry attempts
    fn max_retries(&self) -> u32 {
        3
    }

    /// Exponential backoff base (milliseconds)
    fn backoff_base_ms(&self) -> u64 {
        100
    }

    /// Health check status
    fn health_check(&self) -> HealthStatus {
        HealthStatus::Healthy
    }

    /// Worker name for logging/monitoring
    fn name(&self) -> &str {
        "UnnamedWorker"
    }

    /// Can this worker be retried on failure?
    fn is_retryable(&self) -> bool {
        true
    }

    /// Resource requirements
    fn resource_quota(&self) -> ResourceQuota {
        ResourceQuota::default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceQuota {
    pub cpu_percent: f64,      // 0.0-1.0
    pub memory_mb: u64,
    pub io_bandwidth_mbps: u64,
    pub network_bandwidth_mbps: u64,
}

impl Default for ResourceQuota {
    fn default() -> Self {
        ResourceQuota {
            cpu_percent: 0.25,
            memory_mb: 256,
            io_bandwidth_mbps: 100,
            network_bandwidth_mbps: 100,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_ordering() {
        assert!(Priority::Critical > Priority::High);
        assert!(Priority::High > Priority::Normal);
        assert!(Priority::Normal > Priority::Low);
        assert!(Priority::Low > Priority::Background);
    }

    #[test]
    fn test_health_status() {
        assert_eq!(HealthStatus::Healthy, HealthStatus::Healthy);
        assert_ne!(HealthStatus::Healthy, HealthStatus::Unhealthy);
    }
}
