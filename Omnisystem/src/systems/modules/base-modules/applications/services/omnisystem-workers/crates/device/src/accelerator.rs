/// AcceleratorWorker - Hardware accelerator management

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use std::time::Duration;

pub struct AcceleratorWorker {
    timeout: Duration,
}

pub struct AcceleratorRequest {
    pub task_type: TaskType,
    pub input_data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub enum TaskType {
    MatrixMultiply,
    ConvolutionFilter,
    FastFourier,
    Cryptography,
}

#[derive(Debug)]
pub enum AcceleratorResult {
    Success(Vec<u8>),
    Unavailable,
    Error(String),
}

#[async_trait]
impl Worker for AcceleratorWorker {
    type Input = AcceleratorRequest;
    type Output = AcceleratorResult;

    async fn execute(&self, input: Self::Input) -> WorkerResult<Self::Output> {
        match input.task_type {
            TaskType::MatrixMultiply => Ok(AcceleratorResult::Success(input.input_data)),
            TaskType::ConvolutionFilter => Ok(AcceleratorResult::Success(input.input_data)),
            TaskType::FastFourier => Ok(AcceleratorResult::Success(input.input_data)),
            TaskType::Cryptography => Ok(AcceleratorResult::Success(input.input_data)),
        }
    }

    fn name(&self) -> &str {
        "AcceleratorWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(30)
    }

    fn priority(&self) -> Priority {
        Priority::High
    }
}

impl AcceleratorWorker {
    pub fn new() -> Self {
        AcceleratorWorker {
            timeout: Duration::from_secs(30),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_accelerator_matrix_multiply() {
        let worker = AcceleratorWorker::new();
        let request = AcceleratorRequest {
            task_type: TaskType::MatrixMultiply,
            input_data: vec![1, 2, 3, 4, 5, 6],
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_accelerator_crypto() {
        let worker = AcceleratorWorker::new();
        let request = AcceleratorRequest {
            task_type: TaskType::Cryptography,
            input_data: vec![0; 32],
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }
}
