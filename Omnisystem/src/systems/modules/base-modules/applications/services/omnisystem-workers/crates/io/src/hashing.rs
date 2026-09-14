/// HashingWorker - Cryptographic hashing

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use sha2::{Sha256, Digest};
use std::time::Duration;

pub struct HashingWorker;

#[async_trait]
impl Worker for HashingWorker {
    type Input = Vec<u8>;
    type Output = String;

    async fn execute(&self, data: Self::Input) -> WorkerResult<Self::Output> {
        let mut hasher = Sha256::new();
        hasher.update(&data);
        let result = hasher.finalize();
        Ok(format!("{:x}", result))
    }

    fn name(&self) -> &str {
        "HashingWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(10)
    }
}
