/// ThermalWorker - Temperature monitoring

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermalStatus {
    pub cpu_temp: f32,
    pub gpu_temp: f32,
    pub battery_temp: f32,
    pub throttling: bool,
}

pub struct ThermalWorker;

#[async_trait]
impl Worker for ThermalWorker {
    type Input = ();
    type Output = ThermalStatus;

    async fn execute(&self, _: Self::Input) -> WorkerResult<Self::Output> {
        Ok(ThermalStatus {
            cpu_temp: 55.0,
            gpu_temp: 48.0,
            battery_temp: 35.5,
            throttling: false,
        })
    }

    fn name(&self) -> &str {
        "ThermalWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(2)
    }

    fn priority(&self) -> Priority {
        Priority::High
    }
}
