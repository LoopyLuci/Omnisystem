//! Error types

use thiserror::Error as ThisError;

#[derive(Debug, Clone, ThisError)]
pub enum Error {
    /// A control id was referenced that has not been defined.
    #[error("unknown control: {0}")]
    UnknownControl(String),
    /// Generic catch-all.
    #[error("{0}")]
    Other(String),
}

/// Result type
pub type Result<T> = std::result::Result<T, Error>;
