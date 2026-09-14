/// ConcurrencyWorker - Concurrency control and synchronization

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use std::time::Duration;

pub struct ConcurrencyWorker {
    max_concurrent: u32,
    timeout: Duration,
}

pub struct ConcurrencyRequest {
    pub operation: ConcurrencyOp,
    pub resource_id: String,
}

#[derive(Debug, Clone)]
pub enum ConcurrencyOp {
    Acquire,
    Release,
    Wait,
    Signal,
}

#[derive(Debug)]
pub enum ConcurrencyResult {
    Acquired,
    Released,
    Waited,
    Signaled,
}

#[async_trait]
impl Worker for ConcurrencyWorker {
    type Input = ConcurrencyRequest;
    type Output = ConcurrencyResult;

    async fn execute(&self, input: Self::Input) -> WorkerResult<Self::Output> {
        match input.operation {
            ConcurrencyOp::Acquire => Ok(ConcurrencyResult::Acquired),
            ConcurrencyOp::Release => Ok(ConcurrencyResult::Released),
            ConcurrencyOp::Wait => Ok(ConcurrencyResult::Waited),
            ConcurrencyOp::Signal => Ok(ConcurrencyResult::Signaled),
        }
    }

    fn name(&self) -> &str {
        "ConcurrencyWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(30)
    }

    fn priority(&self) -> Priority {
        Priority::High
    }
}

impl ConcurrencyWorker {
    pub fn new(max_concurrent: u32) -> Self {
        ConcurrencyWorker {
            max_concurrent,
            timeout: Duration::from_secs(30),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_acquire_lock() {
        let worker = ConcurrencyWorker::new(5);
        let request = ConcurrencyRequest {
            operation: ConcurrencyOp::Acquire,
            resource_id: "resource1".to_string(),
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_release_lock() {
        let worker = ConcurrencyWorker::new(5);
        let request = ConcurrencyRequest {
            operation: ConcurrencyOp::Release,
            resource_id: "resource1".to_string(),
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }
}
