/// AudioWorker - Audio playback and recording

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use std::time::Duration;

pub struct AudioWorker;

pub enum AudioOperation {
    Play(String),
    Record(u32), // duration in seconds
}

#[async_trait]
impl Worker for AudioWorker {
    type Input = AudioOperation;
    type Output = String;

    async fn execute(&self, op: Self::Input) -> WorkerResult<Self::Output> {
        match op {
            AudioOperation::Play(file) => Ok(format!("Playing {}", file)),
            AudioOperation::Record(duration) => Ok(format!("Recording for {} seconds", duration)),
        }
    }

    fn name(&self) -> &str {
        "AudioWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(30)
    }
}
