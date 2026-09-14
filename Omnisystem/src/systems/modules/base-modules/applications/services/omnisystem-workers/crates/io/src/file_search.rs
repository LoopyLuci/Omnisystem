/// FileSearchWorker - Full-text file search

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use std::path::PathBuf;
use std::time::Duration;

pub struct FileSearchWorker;

pub struct SearchRequest {
    pub directory: PathBuf,
    pub pattern: String,
    pub recursive: bool,
}

#[async_trait]
impl Worker for FileSearchWorker {
    type Input = SearchRequest;
    type Output = Vec<String>;

    async fn execute(&self, request: Self::Input) -> WorkerResult<Self::Output> {
        let mut results = Vec::new();

        if !request.directory.is_dir() {
            return Err(WorkerError::ExecutionFailed("Not a directory".to_string()));
        }

        // Simple implementation - would be more sophisticated in production
        let pattern = regex::Regex::new(&request.pattern)
            .map_err(|e| WorkerError::ExecutionFailed(e.to_string()))?;

        match tokio::fs::read_dir(&request.directory).await {
            Ok(mut entries) => {
                while let Ok(Some(entry)) = entries.next_entry().await {
                    if let Ok(metadata) = entry.metadata().await {
                        if metadata.is_file() {
                            if let Some(path) = entry.path().to_str() {
                                if pattern.is_match(path) {
                                    results.push(path.to_string());
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => return Err(WorkerError::ExecutionFailed(e.to_string())),
        }

        Ok(results)
    }

    fn name(&self) -> &str {
        "FileSearchWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(60)
    }
}
