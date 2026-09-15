//! Error types

use thiserror::Error as ThisError;

#[derive(Debug, Clone, ThisError)]
pub enum Error {
    #[error("record not found: {0}")]
    NotFound(String),
    #[error("standby environment is not healthy; cannot promote")]
    StandbyNotHealthy,
    #[error("no previous active environment to roll back to")]
    NoRollbackTarget,
    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, Error>;
