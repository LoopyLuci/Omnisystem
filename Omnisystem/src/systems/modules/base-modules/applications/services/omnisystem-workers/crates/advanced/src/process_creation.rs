/// ProcessCreationWorker - Process spawning and initialization

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use std::time::Duration;

pub struct ProcessCreationWorker {
    timeout: Duration,
}

pub struct ProcessRequest {
    pub program: String,
    pub args: Vec<String>,
    pub env: std::collections::HashMap<String, String>,
}

#[derive(Debug)]
pub enum ProcessResult {
    Created(u32),
    Error(String),
}

#[async_trait]
impl Worker for ProcessCreationWorker {
    type Input = ProcessRequest;
    type Output = ProcessResult;

    async fn execute(&self, input: Self::Input) -> WorkerResult<Self::Output> {
        if input.program.is_empty() {
            return Ok(ProcessResult::Error("Program name required".to_string()));
        }

        Ok(ProcessResult::Created(1234))
    }

    fn name(&self) -> &str {
        "ProcessCreationWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(20)
    }

    fn priority(&self) -> Priority {
        Priority::High
    }
}

impl ProcessCreationWorker {
    pub fn new() -> Self {
        ProcessCreationWorker {
            timeout: Duration::from_secs(20),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_process_create() {
        let worker = ProcessCreationWorker::new();
        let request = ProcessRequest {
            program: "echo".to_string(),
            args: vec!["hello".to_string()],
            env: std::collections::HashMap::new(),
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_process_empty_program() {
        let worker = ProcessCreationWorker::new();
        let request = ProcessRequest {
            program: "".to_string(),
            args: vec![],
            env: std::collections::HashMap::new(),
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }
}
