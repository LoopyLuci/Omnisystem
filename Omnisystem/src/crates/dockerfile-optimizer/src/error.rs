//! Error types

/// Errors produced by Dockerfile parsing.
#[derive(Debug, Clone)]
pub enum Error {
    /// A line continuation (`\`) was the last line of the file, with
    /// nothing to continue onto.
    DanglingContinuation(usize),
    /// Other error
    Other(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::DanglingContinuation(line) => write!(f, "dangling line continuation at line {}", line),
            Error::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

/// Result type
pub type Result<T> = std::result::Result<T, Error>;
