//! Error types

use thiserror::Error as ThisError;

#[derive(Debug, Clone, ThisError)]
pub enum Error {
    /// A role name was referenced that has not been defined.
    #[error("unknown role: {0}")]
    UnknownRole(String),
    /// A permission id was referenced that has not been defined.
    #[error("unknown permission: {0}")]
    UnknownPermission(String),
    /// A role hierarchy edge would create a cycle (role inherits from itself
    /// transitively).
    #[error("role hierarchy cycle detected involving role: {0}")]
    HierarchyCycle(String),
    /// Generic catch-all.
    #[error("{0}")]
    Other(String),
}

/// Result type
pub type Result<T> = std::result::Result<T, Error>;
