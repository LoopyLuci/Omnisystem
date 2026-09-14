/// DirectoryWorker - Directory enumeration

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use std::path::PathBuf;
use std::time::Duration;

pub struct DirectoryWorker;

#[async_trait]
impl Worker for DirectoryWorker {
    type Input = PathBuf;
    type Output = Vec<String>;

    async fn execute(&self, path: Self::Input) -> WorkerResult<Self::Output> {
        let mut entries = Vec::new();

        match tokio::fs::read_dir(&path).await {
            Ok(mut dir) => {
                while let Ok(Some(entry)) = dir.next_entry().await {
                    if let Some(path) = entry.path().to_str() {
                        entries.push(path.to_string());
                    }
                }
            }
            Err(e) => return Err(WorkerError::ExecutionFailed(e.to_string())),
        }

        Ok(entries)
    }

    fn name(&self) -> &str {
        "DirectoryWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(30)
    }
}
