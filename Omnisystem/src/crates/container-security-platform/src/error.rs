//! Error types

use thiserror::Error as ThisError;

#[derive(Debug, Clone, ThisError)]
pub enum Error {
    /// Attempted to build a policy gate with an invalid severity threshold.
    #[error("invalid severity string: {0}")]
    InvalidSeverity(String),
    /// Generic catch-all.
    #[error("{0}")]
    Other(String),
}

/// Result type
pub type Result<T> = std::result::Result<T, Error>;
