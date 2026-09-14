/// SensorWorker - Hardware sensor reading

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorData {
    pub accelerometer: (f32, f32, f32),
    pub gyroscope: (f32, f32, f32),
    pub magnetometer: (f32, f32, f32),
}

pub struct SensorWorker;

#[async_trait]
impl Worker for SensorWorker {
    type Input = ();
    type Output = SensorData;

    async fn execute(&self, _: Self::Input) -> WorkerResult<Self::Output> {
        Ok(SensorData {
            accelerometer: (0.1, 0.05, 9.8),
            gyroscope: (0.0, 0.0, 0.0),
            magnetometer: (25.0, -15.0, 40.0),
        })
    }

    fn name(&self) -> &str {
        "SensorWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(1)
    }

    fn priority(&self) -> Priority {
        Priority::Normal
    }
}
