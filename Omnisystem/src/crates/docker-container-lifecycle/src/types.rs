//! Data types
use serde::{Deserialize, Serialize};

/// Container lifecycle state, following the real Docker container states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContainerState {
    /// Container created but never started.
    Created,
    /// Container is running.
    Running,
    /// Container is running but paused (process frozen).
    Paused,
    /// Container process exited (or was stopped) but the container still exists.
    Stopped,
    /// Container has been removed and no longer exists.
    Removed,
}

impl std::fmt::Display for ContainerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ContainerState::Created => "created",
            ContainerState::Running => "running",
            ContainerState::Paused => "paused",
            ContainerState::Stopped => "stopped",
            ContainerState::Removed => "removed",
        };
        write!(f, "{s}")
    }
}

/// A tracked container and its current lifecycle state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Container {
    /// Container id/name.
    pub id: String,
    /// Current lifecycle state.
    pub state: ContainerState,
}
