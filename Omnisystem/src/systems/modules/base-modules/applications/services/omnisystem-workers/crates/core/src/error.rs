/// Error Types

use thiserror::Error;

#[derive(Error, Debug)]
pub enum WorkerError {
    #[error("Execution timeout")]
    Timeout,

    #[error("Task cancelled")]
    Cancelled,

    #[error("Worker unhealthy")]
    UnhealthyWorker,

    #[error("Resource exhausted")]
    ResourceExhausted,

    #[error("Task failed: {0}")]
    ExecutionFailed(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Anyhow error: {0}")]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, WorkerError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_types() {
        let timeout_err = WorkerError::Timeout;
        assert_eq!(timeout_err.to_string(), "Execution timeout");
    }
}
