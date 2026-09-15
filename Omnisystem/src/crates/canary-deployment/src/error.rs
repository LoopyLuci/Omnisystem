//! Error types

use thiserror::Error as ThisError;

#[derive(Debug, Clone, ThisError)]
pub enum Error {
    #[error("record not found: {0}")]
    NotFound(String),
    #[error("no canary rollout is currently in progress")]
    NotStarted,
    #[error("a canary rollout is already in progress")]
    AlreadyStarted,
    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, Error>;
