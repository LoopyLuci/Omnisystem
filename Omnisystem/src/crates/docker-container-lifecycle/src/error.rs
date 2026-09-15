//! Error types
use crate::types::ContainerState;

/// Errors produced by container lifecycle operations.
#[derive(Debug, Clone)]
pub enum Error {
    /// No container is tracked under this id.
    UnknownContainer(String),
    /// The requested transition isn't valid from the container's current state.
    InvalidTransition {
        /// The container id.
        id: String,
        /// The state the container was in.
        from: ContainerState,
        /// The state that was requested.
        to: ContainerState,
    },
    /// A container with this id is already tracked.
    AlreadyExists(String),
    /// Other error
    Other(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::UnknownContainer(id) => write!(f, "no container tracked with id '{}'", id),
            Error::InvalidTransition { id, from, to } => {
                write!(f, "container '{}' cannot transition from {} to {}", id, from, to)
            }
            Error::AlreadyExists(id) => write!(f, "container '{}' already exists", id),
            Error::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

/// Result type
pub type Result<T> = std::result::Result<T, Error>;
