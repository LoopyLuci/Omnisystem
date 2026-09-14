/// JSONParseWorker - JSON parsing and validation

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use std::time::Duration;

pub struct JSONParseWorker;

#[async_trait]
impl Worker for JSONParseWorker {
    type Input = String;
    type Output = serde_json::Value;

    async fn execute(&self, json: Self::Input) -> WorkerResult<Self::Output> {
        serde_json::from_str(&json)
            .map_err(|e| WorkerError::ExecutionFailed(e.to_string()))
    }

    fn name(&self) -> &str {
        "JSONParseWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(5)
    }
}
