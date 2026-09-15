//! Error types

/// Errors produced by image reference parsing and image-manager operations.
#[derive(Debug, Clone)]
pub enum Error {
    /// The image reference string could not be parsed.
    InvalidReference(String),
    /// No image is tracked under this reference.
    ImageNotFound(String),
    /// An image with this exact reference is already tracked.
    AlreadyExists(String),
    /// Other error
    Other(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidReference(s) => write!(f, "invalid image reference: '{}'", s),
            Error::ImageNotFound(s) => write!(f, "image not found: '{}'", s),
            Error::AlreadyExists(s) => write!(f, "image already exists: '{}'", s),
            Error::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

/// Result type
pub type Result<T> = std::result::Result<T, Error>;
