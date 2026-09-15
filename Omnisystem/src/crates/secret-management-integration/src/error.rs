//! Error types

use thiserror::Error as ThisError;

#[derive(Debug, Clone, ThisError)]
pub enum Error {
    /// A secret name was referenced that has not been registered.
    #[error("unknown secret: {0}")]
    UnknownSecret(String),
    /// A grant was referenced that does not exist for the given secret.
    #[error("no such access grant for {secret}: principal {principal}")]
    UnknownGrant {
        /// Secret name.
        secret: String,
        /// Principal (user/service) name.
        principal: String,
    },
    /// Generic catch-all.
    #[error("{0}")]
    Other(String),
}

/// Result type
pub type Result<T> = std::result::Result<T, Error>;
