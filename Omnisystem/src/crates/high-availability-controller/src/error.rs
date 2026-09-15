//! Error types

#[derive(Debug, Clone)]
pub enum Error {
    /// The node is not a member of the cluster.
    UnknownNode(String),
    /// No quorum of live nodes is available, so no leader can be elected.
    NoQuorum,
    /// Other error
    Other(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::UnknownNode(id) => write!(f, "node '{}' is not a cluster member", id),
            Error::NoQuorum => write!(f, "no quorum of live nodes available"),
            Error::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

/// Result type
pub type Result<T> = std::result::Result<T, Error>;
