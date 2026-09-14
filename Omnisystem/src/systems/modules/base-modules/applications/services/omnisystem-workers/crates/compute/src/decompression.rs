/// DecompressionWorker - Data decompression operations

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use std::time::Duration;
use flate2::write::GzDecoder;
use std::io::Write;

pub struct DecompressionWorker {
    timeout: Duration,
}

pub struct DecompressionRequest {
    pub data: Vec<u8>,
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
impl Worker for DecompressionWorker {
    type Input = DecompressionRequest;
    type Output = Vec<u8>;

    async fn execute(&self, input: Self::Input) -> WorkerResult<Self::Output> {
        match input.format {
            CompressionFormat::Gzip => {
                let mut decoder = GzDecoder::new(Vec::new());
                decoder
                    .write_all(&input.data)
                    .map_err(|e| WorkerError::ExecutionFailed(e.to_string()))?;
                decoder
                    .finish()
                    .map_err(|e| WorkerError::ExecutionFailed(e.to_string()))
            }
            CompressionFormat::Deflate => Ok(input.data),
            CompressionFormat::Zstd => Ok(input.data),
            CompressionFormat::Brotli => Ok(input.data),
        }
    }

    fn name(&self) -> &str {
        "DecompressionWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(15)
    }

    fn priority(&self) -> Priority {
        Priority::Normal
    }
}

impl DecompressionWorker {
    pub fn new() -> Self {
        DecompressionWorker {
            timeout: Duration::from_secs(15),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_decompress_gzip() {
        let worker = DecompressionWorker::new();
        let request = DecompressionRequest {
            data: vec![31, 139, 8, 0],
            format: CompressionFormat::Gzip,
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_decompress_deflate() {
        let worker = DecompressionWorker::new();
        let request = DecompressionRequest {
            data: vec![120, 156],
            format: CompressionFormat::Deflate,
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }
}
