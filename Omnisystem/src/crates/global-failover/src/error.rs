//! Error types

/// Errors produced by failover-group operations.
#[derive(Debug, Clone)]
pub enum Error {
    /// The region is not a member of the failover group.
    UnknownRegion(String),
    /// Other error
    Other(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::UnknownRegion(id) => write!(f, "region '{}' is not a member of the failover group", id),
            Error::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

/// Result type
pub type Result<T> = std::result::Result<T, Error>;
