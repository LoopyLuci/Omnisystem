//! Error types

use thiserror::Error as ThisError;

#[derive(Debug, Clone, ThisError)]
pub enum Error {
    #[error("instance not found: {0}")]
    NotFound(String),
    #[error("instance {0} is not ready to begin draining")]
    NotReady(String),
    #[error("instance {0} cannot be terminated while it still has in-flight requests or is not draining")]
    NotDrainable(String),
    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, Error>;
