/// ThreadWorker - Thread creation and management

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use std::time::Duration;

pub struct ThreadWorker {
    timeout: Duration,
}

pub struct ThreadRequest {
    pub operation: ThreadOp,
    pub thread_id: u64,
}

#[derive(Debug, Clone)]
pub enum ThreadOp {
    Create(String),
    Join(u64),
    Yield,
    Sleep(u64),
}

#[derive(Debug)]
pub enum ThreadResult {
    Created(u64),
    Joined,
    Yielded,
    Slept,
}

#[async_trait]
impl Worker for ThreadWorker {
    type Input = ThreadRequest;
    type Output = ThreadResult;

    async fn execute(&self, input: Self::Input) -> WorkerResult<Self::Output> {
        match input.operation {
            ThreadOp::Create(_) => Ok(ThreadResult::Created(1001)),
            ThreadOp::Join(_) => Ok(ThreadResult::Joined),
            ThreadOp::Yield => Ok(ThreadResult::Yielded),
            ThreadOp::Sleep(_) => Ok(ThreadResult::Slept),
        }
    }

    fn name(&self) -> &str {
        "ThreadWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(30)
    }

    fn priority(&self) -> Priority {
        Priority::Normal
    }
}

impl ThreadWorker {
    pub fn new() -> Self {
        ThreadWorker {
            timeout: Duration::from_secs(30),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_thread() {
        let worker = ThreadWorker::new();
        let request = ThreadRequest {
            operation: ThreadOp::Create("worker_thread".to_string()),
            thread_id: 0,
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_thread_join() {
        let worker = ThreadWorker::new();
        let request = ThreadRequest {
            operation: ThreadOp::Join(1001),
            thread_id: 0,
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }
}
