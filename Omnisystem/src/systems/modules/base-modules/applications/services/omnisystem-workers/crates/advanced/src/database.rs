/// DatabaseWorker - SQL query execution

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use std::time::Duration;

pub struct DatabaseWorker;

pub struct QueryRequest {
    pub sql: String,
}

#[async_trait]
impl Worker for DatabaseWorker {
    type Input = QueryRequest;
    type Output = String;

    async fn execute(&self, request: Self::Input) -> WorkerResult<Self::Output> {
        // Would execute against database
        Ok(format!("Query result for: {}", request.sql))
    }

    fn name(&self) -> &str {
        "DatabaseWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(10)
    }
}
