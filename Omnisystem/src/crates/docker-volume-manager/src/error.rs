//! Error types

/// Errors produced by volume-manager operations.
#[derive(Debug, Clone)]
pub enum Error {
    /// A volume with this name already exists.
    AlreadyExists(String),
    /// No volume is tracked under this name.
    NotFound(String),
    /// The mount point is already in use by another container on this host.
    MountPointConflict {
        /// The conflicting host mount path.
        path: String,
        /// The container currently using that mount path.
        existing_container: String,
    },
    /// Attempted to remove a volume that still has active mounts.
    VolumeInUse(String),
    /// Other error
    Other(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::AlreadyExists(name) => write!(f, "volume '{}' already exists", name),
            Error::NotFound(name) => write!(f, "volume '{}' not found", name),
            Error::MountPointConflict { path, existing_container } => write!(
                f,
                "mount point '{}' is already in use by container '{}'",
                path, existing_container
            ),
            Error::VolumeInUse(name) => write!(f, "volume '{}' is still mounted and cannot be removed", name),
            Error::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

/// Result type
pub type Result<T> = std::result::Result<T, Error>;
