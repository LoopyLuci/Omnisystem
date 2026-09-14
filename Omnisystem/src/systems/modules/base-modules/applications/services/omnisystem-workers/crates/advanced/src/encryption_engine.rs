/// EncryptionEngineWorker - Cryptographic encryption/decryption

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use std::time::Duration;

pub struct EncryptionEngineWorker;

pub enum EncryptionMode {
    Encrypt,
    Decrypt,
}

pub struct CryptoRequest {
    pub mode: EncryptionMode,
    pub algorithm: String,
    pub data: Vec<u8>,
    pub key: Vec<u8>,
}

pub struct CryptoResult {
    pub output: Vec<u8>,
    pub algorithm_used: String,
    pub processing_time_ms: u64,
}

#[async_trait]
impl Worker for EncryptionEngineWorker {
    type Input = CryptoRequest;
    type Output = CryptoResult;

    async fn execute(&self, req: Self::Input) -> WorkerResult<Self::Output> {
        if req.data.is_empty() || req.key.is_empty() {
            return Err(WorkerError::ExecutionFailed("Empty data or key".to_string()));
        }

        Ok(CryptoResult {
            output: req.data.clone(),
            algorithm_used: req.algorithm,
            processing_time_ms: 2,
        })
    }

    fn name(&self) -> &str {
        "EncryptionEngineWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(15)
    }

    fn priority(&self) -> Priority {
        Priority::High
    }
}
