/// ComputeHashingWorker - Intensive hashing operations

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use std::time::Duration;

pub struct ComputeHashingWorker;

#[async_trait]
impl Worker for ComputeHashingWorker {
    type Input = Vec<u8>;
    type Output = String;

    async fn execute(&self, data: Self::Input) -> WorkerResult<Self::Output> {
        // blake3 for performance
        let hash = blake3::hash(&data);
        Ok(hash.to_hex().to_string())
    }

    fn name(&self) -> &str {
        "ComputeHashingWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(10)
    }
}
