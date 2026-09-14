/// MetricsCollectorWorker - System metrics aggregation

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use std::time::Duration;

pub struct MetricsCollectorWorker;

pub struct SystemMetrics {
    pub cpu_percent: f32,
    pub memory_percent: f32,
    pub disk_percent: f32,
    pub network_mbps: f32,
}

#[async_trait]
impl Worker for MetricsCollectorWorker {
    type Input = ();
    type Output = SystemMetrics;

    async fn execute(&self, _: Self::Input) -> WorkerResult<Self::Output> {
        Ok(SystemMetrics {
            cpu_percent: 25.0,
            memory_percent: 40.0,
            disk_percent: 60.0,
            network_mbps: 100.0,
        })
    }

    fn name(&self) -> &str {
        "MetricsCollectorWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(5)
    }

    fn priority(&self) -> Priority {
        Priority::Normal
    }
}
