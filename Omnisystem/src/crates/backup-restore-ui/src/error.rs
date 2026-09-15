//! Error types for backup chain validation and restore planning.

/// Errors produced by this crate.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// Two backups shared the same id.
    DuplicateId(String),
    /// An incremental backup's parent id does not exist.
    MissingParent {
        /// The backup with the dangling parent reference.
        id: String,
        /// The missing parent id.
        parent: String,
    },
    /// Following parent links from a backup did not terminate at a full
    /// backup (a cycle, most likely).
    BrokenChain(String),
    /// The requested backup id was not found.
    NotFound(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::DuplicateId(id) => write!(f, "duplicate backup id '{id}'"),
            Error::MissingParent { id, parent } => {
                write!(f, "backup '{id}' references missing parent '{parent}'")
            }
            Error::BrokenChain(id) => write!(f, "backup chain for '{id}' is broken or cyclic"),
            Error::NotFound(id) => write!(f, "backup '{id}' not found"),
        }
    }
}

impl std::error::Error for Error {}

/// Result type for this crate.
pub type Result<T> = std::result::Result<T, Error>;
