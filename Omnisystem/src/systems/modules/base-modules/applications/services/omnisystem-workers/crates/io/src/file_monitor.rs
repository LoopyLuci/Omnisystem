/// FileMonitorWorker - File system change detection

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use std::path::PathBuf;
use std::time::Duration;
use std::collections::HashMap;

pub struct FileMonitorWorker {
    watch_timeout: Duration,
}

pub struct MonitorRequest {
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub enum MonitorEvent {
    Created,
    Modified,
    Deleted,
    Renamed,
    PermissionChanged,
}

#[async_trait]
impl Worker for FileMonitorWorker {
    type Input = MonitorRequest;
    type Output = MonitorEvent;

    async fn execute(&self, input: Self::Input) -> WorkerResult<Self::Output> {
        if !input.path.exists() {
            return Ok(MonitorEvent::Deleted);
        }

        let metadata = tokio::fs::metadata(&input.path)
            .await
            .map_err(|e| WorkerError::ExecutionFailed(e.to_string()))?;

        if metadata.is_file() {
            Ok(MonitorEvent::Modified)
        } else {
            Ok(MonitorEvent::Created)
        }
    }

    fn name(&self) -> &str {
        "FileMonitorWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(30)
    }

    fn priority(&self) -> Priority {
        Priority::Normal
    }
}

impl FileMonitorWorker {
    pub fn new() -> Self {
        FileMonitorWorker {
            watch_timeout: Duration::from_secs(30),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_monitor_existing_file() {
        let worker = FileMonitorWorker::new();
        let temp_file = std::env::temp_dir().join("test_monitor.txt");
        tokio::fs::write(&temp_file, "test").await.unwrap();

        let request = MonitorRequest { path: temp_file.clone() };
        let result = worker.execute(request).await;
        assert!(result.is_ok());

        tokio::fs::remove_file(temp_file).await.ok();
    }

    #[tokio::test]
    async fn test_monitor_deleted_file() {
        let worker = FileMonitorWorker::new();
        let request = MonitorRequest {
            path: PathBuf::from("/nonexistent/path/file.txt"),
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), MonitorEvent::Deleted));
    }
}
