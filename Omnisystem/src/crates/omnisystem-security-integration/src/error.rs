//! Error types

use thiserror::Error as ThisError;

#[derive(Debug, Clone, ThisError)]
pub enum Error {
    /// A finding id was referenced that was never ingested.
    #[error("unknown finding: {0}")]
    UnknownFinding(String),
    /// Generic catch-all.
    #[error("{0}")]
    Other(String),
}

/// Result type
pub type Result<T> = std::result::Result<T, Error>;
