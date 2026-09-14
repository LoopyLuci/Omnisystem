/// FileWriteWorker - Safe file writing

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use std::path::PathBuf;
use std::time::Duration;

pub struct FileWriteWorker;

pub struct WriteRequest {
    pub path: PathBuf,
    pub data: Vec<u8>,
    pub atomic: bool,
}

#[async_trait]
impl Worker for FileWriteWorker {
    type Input = WriteRequest;
    type Output = ();

    async fn execute(&self, request: Self::Input) -> WorkerResult<Self::Output> {
        if request.atomic {
            let temp_path = request.path.with_extension("tmp");
            tokio::fs::write(&temp_path, &request.data)
                .await
                .map_err(|e| WorkerError::ExecutionFailed(e.to_string()))?;

            tokio::fs::rename(&temp_path, &request.path)
                .await
                .map_err(|e| WorkerError::ExecutionFailed(e.to_string()))?;
        } else {
            tokio::fs::write(&request.path, &request.data)
                .await
                .map_err(|e| WorkerError::ExecutionFailed(e.to_string()))?;
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "FileWriteWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(30)
    }
}
