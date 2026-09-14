/// BufferWorker - Memory buffer management

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use std::time::Duration;

pub struct BufferWorker {
    max_size: usize,
}

pub struct BufferRequest {
    pub data: Vec<u8>,
    pub operation: BufferOp,
}

#[derive(Debug, Clone)]
pub enum BufferOp {
    Allocate(usize),
    Write,
    Read(usize),
    Clear,
}

#[derive(Debug)]
pub enum BufferResult {
    Success(usize),
    Data(Vec<u8>),
    Cleared,
}

#[async_trait]
impl Worker for BufferWorker {
    type Input = BufferRequest;
    type Output = BufferResult;

    async fn execute(&self, input: Self::Input) -> WorkerResult<Self::Output> {
        match input.operation {
            BufferOp::Allocate(size) => {
                if size > self.max_size {
                    Err(WorkerError::ExecutionFailed("Buffer too large".to_string()))
                } else {
                    Ok(BufferResult::Success(size))
                }
            }
            BufferOp::Write => Ok(BufferResult::Success(input.data.len())),
            BufferOp::Read(size) => {
                let read_size = std::cmp::min(size, input.data.len());
                Ok(BufferResult::Data(input.data[..read_size].to_vec()))
            }
            BufferOp::Clear => Ok(BufferResult::Cleared),
        }
    }

    fn name(&self) -> &str {
        "BufferWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(10)
    }

    fn priority(&self) -> Priority {
        Priority::High
    }
}

impl BufferWorker {
    pub fn new(max_size: usize) -> Self {
        BufferWorker { max_size }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_buffer_allocate() {
        let worker = BufferWorker::new(1024);
        let request = BufferRequest {
            data: vec![],
            operation: BufferOp::Allocate(512),
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_buffer_write() {
        let worker = BufferWorker::new(1024);
        let request = BufferRequest {
            data: vec![1, 2, 3, 4, 5],
            operation: BufferOp::Write,
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_buffer_read() {
        let worker = BufferWorker::new(1024);
        let request = BufferRequest {
            data: vec![1, 2, 3, 4, 5],
            operation: BufferOp::Read(3),
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }
}
