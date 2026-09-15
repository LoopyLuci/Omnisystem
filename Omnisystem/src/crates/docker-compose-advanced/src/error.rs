//! Error types

/// Errors produced by compose file operations.
#[derive(Debug, Clone)]
pub enum Error {
    /// A service definition referenced an undefined service name.
    UnknownService(String),
    /// Other error
    Other(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::UnknownService(s) => write!(f, "unknown service: '{}'", s),
            Error::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

/// Result type
pub type Result<T> = std::result::Result<T, Error>;
