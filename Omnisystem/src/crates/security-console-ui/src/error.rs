//! Error types

use thiserror::Error as ThisError;

#[derive(Debug, Clone, ThisError)]
pub enum Error {
    /// Generic catch-all (this crate does pure data shaping and rarely
    /// fails; kept for API consistency with the rest of the cluster).
    #[error("{0}")]
    Other(String),
}

/// Result type
pub type Result<T> = std::result::Result<T, Error>;
