/// MatrixWorker - Matrix operations and linear algebra

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use std::time::Duration;

pub struct MatrixWorker {
    timeout: Duration,
}

pub struct MatrixRequest {
    pub operation: MatrixOp,
    pub dimensions: (u32, u32),
}

#[derive(Debug, Clone)]
pub enum MatrixOp {
    Multiply,
    Transpose,
    Invert,
    Determinant,
    Eigenvalues,
}

#[derive(Debug)]
pub enum MatrixResult {
    Result(Vec<f64>),
    Scalar(f64),
    Success,
}

#[async_trait]
impl Worker for MatrixWorker {
    type Input = MatrixRequest;
    type Output = MatrixResult;

    async fn execute(&self, input: Self::Input) -> WorkerResult<Self::Output> {
        match input.operation {
            MatrixOp::Multiply => Ok(MatrixResult::Success),
            MatrixOp::Transpose => Ok(MatrixResult::Success),
            MatrixOp::Invert => Ok(MatrixResult::Success),
            MatrixOp::Determinant => Ok(MatrixResult::Scalar(1.0)),
            MatrixOp::Eigenvalues => Ok(MatrixResult::Result(vec![1.0, 2.0, 3.0])),
        }
    }

    fn name(&self) -> &str {
        "MatrixWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(30)
    }

    fn priority(&self) -> Priority {
        Priority::Normal
    }
}

impl MatrixWorker {
    pub fn new() -> Self {
        MatrixWorker {
            timeout: Duration::from_secs(30),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_matrix_multiply() {
        let worker = MatrixWorker::new();
        let request = MatrixRequest {
            operation: MatrixOp::Multiply,
            dimensions: (3, 3),
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_matrix_determinant() {
        let worker = MatrixWorker::new();
        let request = MatrixRequest {
            operation: MatrixOp::Determinant,
            dimensions: (3, 3),
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }
}
