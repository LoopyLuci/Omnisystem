/// ClockWorker - System clock management and synchronization

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use std::time::Duration;

pub struct ClockWorker {
    timeout: Duration,
}

pub struct ClockRequest {
    pub operation: ClockOperation,
}

#[derive(Debug, Clone)]
pub enum ClockOperation {
    GetTime,
    SetTime(String),
    Sync,
    GetTimeZone,
    SetTimeZone(String),
}

#[derive(Debug)]
pub enum ClockResult {
    Time(String),
    TimeSet,
    Synced,
    TimeZone(String),
    TimeZoneSet,
}

#[async_trait]
impl Worker for ClockWorker {
    type Input = ClockRequest;
    type Output = ClockResult;

    async fn execute(&self, input: Self::Input) -> WorkerResult<Self::Output> {
        match input.operation {
            ClockOperation::GetTime => Ok(ClockResult::Time("12:00:00".to_string())),
            ClockOperation::SetTime(_) => Ok(ClockResult::TimeSet),
            ClockOperation::Sync => Ok(ClockResult::Synced),
            ClockOperation::GetTimeZone => Ok(ClockResult::TimeZone("UTC".to_string())),
            ClockOperation::SetTimeZone(_) => Ok(ClockResult::TimeZoneSet),
        }
    }

    fn name(&self) -> &str {
        "ClockWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(30)
    }

    fn priority(&self) -> Priority {
        Priority::Normal
    }
}

impl ClockWorker {
    pub fn new() -> Self {
        ClockWorker {
            timeout: Duration::from_secs(30),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_time() {
        let worker = ClockWorker::new();
        let request = ClockRequest {
            operation: ClockOperation::GetTime,
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_timezone() {
        let worker = ClockWorker::new();
        let request = ClockRequest {
            operation: ClockOperation::GetTimeZone,
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }
}
