/// DisplayWorker - Screen control and rendering

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use std::time::Duration;

pub struct DisplayWorker;

pub struct DisplayCommand {
    pub brightness: u8,
    pub refresh_rate: u16,
}

#[async_trait]
impl Worker for DisplayWorker {
    type Input = DisplayCommand;
    type Output = String;

    async fn execute(&self, cmd: Self::Input) -> WorkerResult<Self::Output> {
        Ok(format!(
            "Display: brightness={}, refresh_rate={}Hz",
            cmd.brightness, cmd.refresh_rate
        ))
    }

    fn name(&self) -> &str {
        "DisplayWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(5)
    }
}
