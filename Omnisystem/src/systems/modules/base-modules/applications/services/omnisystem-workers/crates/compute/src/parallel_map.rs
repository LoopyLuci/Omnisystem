/// ParallelMapWorker - Parallel map/reduce operations

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use std::time::Duration;

pub struct ParallelMapWorker;

#[async_trait]
impl Worker for ParallelMapWorker {
    type Input = Vec<i64>;
    type Output = Vec<i64>;

    async fn execute(&self, data: Self::Input) -> WorkerResult<Self::Output> {
        let result = tokio::task::spawn_blocking(move || {
            data.iter()
                .map(|x| x * 2)
                .collect::<Vec<_>>()
        })
        .await
        .map_err(|e| WorkerError::ExecutionFailed(e.to_string()))?;

        Ok(result)
    }

    fn name(&self) -> &str {
        "ParallelMapWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(60)
    }

    fn priority(&self) -> Priority {
        Priority::Normal
    }
}
