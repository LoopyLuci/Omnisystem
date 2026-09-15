//! Error types

#[derive(Debug, Clone)]
pub enum Error {
    /// No check is registered under this name.
    UnknownCheck(String),
    /// Other error
    Other(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::UnknownCheck(name) => write!(f, "no health check registered as '{}'", name),
            Error::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

/// Result type
pub type Result<T> = std::result::Result<T, Error>;
