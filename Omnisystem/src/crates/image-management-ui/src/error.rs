//! Error types for image registry management.

/// Errors produced by this crate.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// The same `(repository, tag)` pair appeared more than once.
    DuplicateTag {
        /// Repository name.
        repository: String,
        /// Tag name.
        tag: String,
    },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::DuplicateTag { repository, tag } => {
                write!(f, "duplicate tag '{repository}:{tag}'")
            }
        }
    }
}

impl std::error::Error for Error {}

/// Result type for this crate.
pub type Result<T> = std::result::Result<T, Error>;
