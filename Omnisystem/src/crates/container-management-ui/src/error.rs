//! Error types for container lifecycle management.

use crate::ContainerState;

/// Errors produced by this crate.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// Two containers were registered under the same name.
    DuplicateContainer(String),
    /// The named container is not registered.
    UnknownContainer(String),
    /// A requested state transition is not legal from the current state.
    IllegalTransition {
        /// Container name.
        name: String,
        /// State it was in.
        from: ContainerState,
        /// State requested.
        to: ContainerState,
    },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::DuplicateContainer(name) => write!(f, "container '{name}' already registered"),
            Error::UnknownContainer(name) => write!(f, "container '{name}' not found"),
            Error::IllegalTransition { name, from, to } => {
                write!(f, "container '{name}' cannot go from {from:?} to {to:?}")
            }
        }
    }
}

impl std::error::Error for Error {}

/// Result type for this crate.
pub type Result<T> = std::result::Result<T, Error>;
