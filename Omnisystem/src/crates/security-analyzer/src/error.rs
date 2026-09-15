//! Error types

use thiserror::Error as ThisError;

#[derive(Debug, Clone, ThisError)]
pub enum Error {
    /// A rule id was referenced that has not been registered.
    #[error("unknown rule: {0}")]
    UnknownRule(String),
    /// Generic catch-all.
    #[error("{0}")]
    Other(String),
}

/// Result type
pub type Result<T> = std::result::Result<T, Error>;
