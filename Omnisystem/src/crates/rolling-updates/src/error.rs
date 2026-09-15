//! Error types

#[derive(Debug, Clone)]
pub enum Error {
    /// Invalid controller configuration.
    InvalidConfig(String),
    /// A batch cannot be started right now (paused, or one already in flight).
    BatchNotStartable(String),
    /// There is no in-flight batch to complete or roll back.
    NoBatchInProgress,
    /// Other error
    Other(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidConfig(msg) => write!(f, "invalid config: {}", msg),
            Error::BatchNotStartable(msg) => write!(f, "batch not startable: {}", msg),
            Error::NoBatchInProgress => write!(f, "no batch is currently in progress"),
            Error::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

/// Result type
pub type Result<T> = std::result::Result<T, Error>;
