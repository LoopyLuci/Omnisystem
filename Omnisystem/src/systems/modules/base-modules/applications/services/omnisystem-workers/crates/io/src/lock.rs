/// LockWorker - Distributed locking and synchronization

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use std::time::Duration;

pub struct LockWorker {
    timeout: Duration,
}

pub struct LockRequest {
    pub resource: String,
    pub operation: LockOp,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone)]
pub enum LockOp {
    Acquire,
    Release,
    Renew,
    Check,
}

#[derive(Debug)]
pub enum LockResult {
    Acquired,
    Released,
    Renewed,
    Held,
}

#[async_trait]
impl Worker for LockWorker {
    type Input = LockRequest;
    type Output = LockResult;

    async fn execute(&self, input: Self::Input) -> WorkerResult<Self::Output> {
        match input.operation {
            LockOp::Acquire => Ok(LockResult::Acquired),
            LockOp::Release => Ok(LockResult::Released),
            LockOp::Renew => Ok(LockResult::Renewed),
            LockOp::Check => Ok(LockResult::Held),
        }
    }

    fn name(&self) -> &str {
        "LockWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(30)
    }

    fn priority(&self) -> Priority {
        Priority::High
    }
}

impl LockWorker {
    pub fn new() -> Self {
        LockWorker {
            timeout: Duration::from_secs(30),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_acquire_lock() {
        let worker = LockWorker::new();
        let request = LockRequest {
            resource: "resource1".to_string(),
            operation: LockOp::Acquire,
            timeout_ms: 5000,
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_release_lock() {
        let worker = LockWorker::new();
        let request = LockRequest {
            resource: "resource1".to_string(),
            operation: LockOp::Release,
            timeout_ms: 0,
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }
}
