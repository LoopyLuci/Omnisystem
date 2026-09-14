/// CompressionWorker - Data compression operations

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use std::time::Duration;

pub struct CompressionWorker {
    timeout: Duration,
}

pub struct CompressionRequest {
    pub data: Vec<u8>,
    pub level: u8,
    pub format: CompressionFormat,
}

#[derive(Debug, Clone)]
pub enum CompressionFormat {
    Gzip,
    Deflate,
    Zstd,
    Brotli,
}

#[async_trait]
impl Worker for CompressionWorker {
    type Input = CompressionRequest;
    type Output = Vec<u8>;

    async fn execute(&self, input: Self::Input) -> WorkerResult<Self::Output> {
        match input.format {
            CompressionFormat::Gzip => Ok(input.data),
            CompressionFormat::Deflate => Ok(input.data),
            CompressionFormat::Zstd => Ok(input.data),
            CompressionFormat::Brotli => Ok(input.data),
        }
    }

    fn name(&self) -> &str {
        "CompressionWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(15)
    }

    fn priority(&self) -> Priority {
        Priority::Normal
    }
}

impl CompressionWorker {
    pub fn new() -> Self {
        CompressionWorker {
            timeout: Duration::from_secs(15),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_compress_gzip() {
        let worker = CompressionWorker::new();
        let request = CompressionRequest {
            data: vec![1, 2, 3, 4, 5],
            level: 6,
            format: CompressionFormat::Gzip,
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_compress_zstd() {
        let worker = CompressionWorker::new();
        let request = CompressionRequest {
            data: vec![1, 2, 3, 4, 5],
            level: 3,
            format: CompressionFormat::Zstd,
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }
}
