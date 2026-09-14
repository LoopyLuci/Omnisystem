/// FileReadWorker - Sequential file reading

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use std::path::PathBuf;
use std::time::Duration;

pub struct FileReadWorker {
    buffer_size: usize,
}

#[async_trait]
impl Worker for FileReadWorker {
    type Input = PathBuf;
    type Output = Vec<u8>;

    async fn execute(&self, path: Self::Input) -> WorkerResult<Self::Output> {
        tokio::fs::read(&path)
            .await
            .map_err(|e| WorkerError::ExecutionFailed(e.to_string()))
    }

    fn name(&self) -> &str {
        "FileReadWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(30)
    }
}

impl FileReadWorker {
    pub fn new(buffer_size: usize) -> Self {
        FileReadWorker { buffer_size }
    }
}
