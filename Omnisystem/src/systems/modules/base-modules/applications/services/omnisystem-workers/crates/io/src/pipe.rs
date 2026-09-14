/// PipeWorker - Inter-process pipe communication

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use std::time::Duration;

pub struct PipeWorker;

pub enum PipeOperation {
    Create(String),      // named pipe
    Write(String, Vec<u8>),
    Read(String, usize),
}

pub enum PipeResult {
    Created(String),
    Written(usize),
    Data(Vec<u8>),
}

#[async_trait]
impl Worker for PipeWorker {
    type Input = PipeOperation;
    type Output = PipeResult;

    async fn execute(&self, op: Self::Input) -> WorkerResult<Self::Output> {
        match op {
            PipeOperation::Create(name) => {
                Ok(PipeResult::Created(format!("pipe://{}", name)))
            },
            PipeOperation::Write(_name, data) => {
                Ok(PipeResult::Written(data.len()))
            },
            PipeOperation::Read(_name, size) => {
                Ok(PipeResult::Data(vec![0u8; size]))
            }
        }
    }

    fn name(&self) -> &str {
        "PipeWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(10)
    }
}
