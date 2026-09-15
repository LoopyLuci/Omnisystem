//! Container management types: lifecycle state and resource limits.

use serde::{Deserialize, Serialize};

/// Lifecycle state of a managed container.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContainerState {
    /// Container has been created but not started.
    Created,
    /// Container is running.
    Running,
    /// Container is running but paused.
    Paused,
    /// Container exited cleanly.
    Stopped,
    /// Container exited with a failure.
    Failed,
}

/// Resource limits and current usage for a container, both in the same
/// arbitrary unit (e.g. MB for memory, millicores for CPU).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// CPU limit.
    pub cpu_limit: f64,
    /// Current CPU usage.
    pub cpu_used: f64,
    /// Memory limit.
    pub memory_limit: f64,
    /// Current memory usage.
    pub memory_used: f64,
}

/// A managed container.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Container {
    /// Container name/id.
    pub name: String,
    /// Current lifecycle state.
    pub state: ContainerState,
    /// Resource limits and usage.
    pub resources: ResourceLimits,
}
