/// SortWorker - Efficient sorting algorithms

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use std::time::Duration;

pub struct SortWorker;

#[async_trait]
impl Worker for SortWorker {
    type Input = Vec<i64>;
    type Output = Vec<i64>;

    async fn execute(&self, mut data: Self::Input) -> WorkerResult<Self::Output> {
        // Run sorting on a blocking thread to not block async runtime
        tokio::task::spawn_blocking(move || {
            data.sort_unstable();
            data
        })
        .await
        .map_err(|e| WorkerError::ExecutionFailed(e.to_string()))
    }

    fn name(&self) -> &str {
        "SortWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(30)
    }
}
