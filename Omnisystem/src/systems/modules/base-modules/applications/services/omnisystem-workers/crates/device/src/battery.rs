/// BatteryWorker - Battery status and charging

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatteryStatus {
    pub percentage: u8,
    pub charging: bool,
    pub temperature: f32,
    pub health: String,
}

pub struct BatteryWorker;

#[async_trait]
impl Worker for BatteryWorker {
    type Input = ();
    type Output = BatteryStatus;

    async fn execute(&self, _: Self::Input) -> WorkerResult<Self::Output> {
        // Would read actual battery status from system
        Ok(BatteryStatus {
            percentage: 85,
            charging: false,
            temperature: 35.5,
            health: "Good".to_string(),
        })
    }

    fn name(&self) -> &str {
        "BatteryWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(2)
    }

    fn priority(&self) -> Priority {
        Priority::High
    }
}
