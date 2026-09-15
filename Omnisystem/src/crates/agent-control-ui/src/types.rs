//! Agent control types.

use serde::{Deserialize, Serialize};

/// Lifecycle state of a controlled agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentState {
    /// Registered but not yet started.
    Idle,
    /// Actively executing.
    Running,
    /// Started, then paused.
    Paused,
    /// Terminated; no further commands are accepted.
    Stopped,
}

/// A command issued from the control panel to an agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Command {
    /// `Idle` -> `Running`.
    Start,
    /// `Running` -> `Paused`.
    Pause,
    /// `Paused` -> `Running`.
    Resume,
    /// `Running`/`Paused`/`Idle` -> `Stopped`.
    Stop,
}

/// A registered agent and its current state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentInfo {
    /// Unique agent id.
    pub id: String,
    /// Human-readable label.
    pub name: String,
    /// Current lifecycle state.
    pub state: AgentState,
    /// Number of commands successfully applied.
    pub commands_applied: u32,
}
