/// RegexWorker - Regular expression matching

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use std::time::Duration;

pub struct RegexWorker;

pub struct RegexRequest {
    pub pattern: String,
    pub text: String,
}

#[async_trait]
impl Worker for RegexWorker {
    type Input = RegexRequest;
    type Output = Vec<String>;

    async fn execute(&self, request: Self::Input) -> WorkerResult<Self::Output> {
        let regex = regex::Regex::new(&request.pattern)
            .map_err(|e| WorkerError::ExecutionFailed(e.to_string()))?;

        let matches: Vec<String> = regex
            .find_iter(&request.text)
            .map(|m| m.as_str().to_string())
            .collect();

        Ok(matches)
    }

    fn name(&self) -> &str {
        "RegexWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(5)
    }
}
