//! Error types for the agent control panel.

/// Errors that can occur while controlling registered agents.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// A command referenced an agent id that isn't registered.
    UnknownAgent(String),
    /// An agent id was registered more than once.
    DuplicateAgent(String),
    /// A command isn't valid from the agent's current state
    /// (e.g. `Resume` while `Idle`, `Pause` while `Stopped`).
    InvalidTransition {
        /// Agent id the command targeted.
        agent_id: String,
        /// The agent's state at the time of the command.
        from: crate::types::AgentState,
        /// The command that was rejected.
        command: crate::types::Command,
    },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::UnknownAgent(id) => write!(f, "unknown agent: {id}"),
            Error::DuplicateAgent(id) => write!(f, "agent already registered: {id}"),
            Error::InvalidTransition { agent_id, from, command } => {
                write!(f, "agent {agent_id}: command {command:?} invalid from state {from:?}")
            }
        }
    }
}

impl std::error::Error for Error {}

/// Result type for agent control operations.
pub type Result<T> = std::result::Result<T, Error>;
