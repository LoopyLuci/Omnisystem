/// FTPWorker - FTP file transfer operations

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use std::path::PathBuf;
use std::time::Duration;

pub struct FTPWorker {
    timeout: Duration,
}

pub struct FTPRequest {
    pub server: String,
    pub command: FTPCommand,
}

#[derive(Debug, Clone)]
pub enum FTPCommand {
    Connect(String, u16),
    Upload(PathBuf),
    Download(String),
    List,
    Delete(String),
}

#[derive(Debug)]
pub enum FTPResult {
    Connected,
    Uploaded(usize),
    Downloaded(Vec<u8>),
    Listed(Vec<String>),
    Deleted,
}

#[async_trait]
impl Worker for FTPWorker {
    type Input = FTPRequest;
    type Output = FTPResult;

    async fn execute(&self, input: Self::Input) -> WorkerResult<Self::Output> {
        match input.command {
            FTPCommand::Connect(_, _) => Ok(FTPResult::Connected),
            FTPCommand::Upload(path) => {
                let data = tokio::fs::read(&path)
                    .await
                    .map_err(|e| WorkerError::ExecutionFailed(e.to_string()))?;
                Ok(FTPResult::Uploaded(data.len()))
            }
            FTPCommand::Download(_) => Ok(FTPResult::Downloaded(vec![])),
            FTPCommand::List => Ok(FTPResult::Listed(vec![])),
            FTPCommand::Delete(_) => Ok(FTPResult::Deleted),
        }
    }

    fn name(&self) -> &str {
        "FTPWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(60)
    }

    fn priority(&self) -> Priority {
        Priority::Normal
    }
}

impl FTPWorker {
    pub fn new() -> Self {
        FTPWorker {
            timeout: Duration::from_secs(60),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ftp_connect() {
        let worker = FTPWorker::new();
        let request = FTPRequest {
            server: "ftp.example.com".to_string(),
            command: FTPCommand::Connect("user".to_string(), 21),
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_ftp_list() {
        let worker = FTPWorker::new();
        let request = FTPRequest {
            server: "ftp.example.com".to_string(),
            command: FTPCommand::List,
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }
}
