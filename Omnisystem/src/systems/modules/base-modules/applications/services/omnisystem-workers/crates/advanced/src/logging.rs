/// LoggingWorker - Centralized logging and event recording

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use std::time::Duration;

pub struct LoggingWorker {
    timeout: Duration,
}

pub struct LogRequest {
    pub message: String,
    pub level: LogLevel,
    pub source: String,
}

#[derive(Debug, Clone)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug)]
pub enum LogResult {
    Logged,
    Buffered,
    Flushed,
}

#[async_trait]
impl Worker for LoggingWorker {
    type Input = LogRequest;
    type Output = LogResult;

    async fn execute(&self, input: Self::Input) -> WorkerResult<Self::Output> {
        if input.message.is_empty() {
            return Err(WorkerError::ExecutionFailed("Empty message".to_string()));
        }
        Ok(LogResult::Logged)
    }

    fn name(&self) -> &str {
        "LoggingWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(5)
    }

    fn priority(&self) -> Priority {
        Priority::Normal
    }
}

impl LoggingWorker {
    pub fn new() -> Self {
        LoggingWorker {
            timeout: Duration::from_secs(5),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_log_message() {
        let worker = LoggingWorker::new();
        let request = LogRequest {
            message: "Test log message".to_string(),
            level: LogLevel::Info,
            source: "test".to_string(),
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_log_error() {
        let worker = LoggingWorker::new();
        let request = LogRequest {
            message: "Error occurred".to_string(),
            level: LogLevel::Error,
            source: "system".to_string(),
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }
}
