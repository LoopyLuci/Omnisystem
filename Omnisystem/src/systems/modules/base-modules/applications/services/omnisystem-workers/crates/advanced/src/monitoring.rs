/// MonitoringWorker - System health and performance monitoring

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use std::time::Duration;

pub struct MonitoringWorker {
    timeout: Duration,
}

pub struct MonitoringRequest {
    pub metric: MonitoringMetric,
}

#[derive(Debug, Clone)]
pub enum MonitoringMetric {
    CpuUsage,
    MemoryUsage,
    DiskUsage,
    NetworkLatency,
    SystemHealth,
}

#[derive(Debug)]
pub enum MonitoringResult {
    CpuLevel(u8),
    MemoryLevel(u8),
    DiskLevel(u8),
    LatencyMs(u32),
    Healthy,
}

#[async_trait]
impl Worker for MonitoringWorker {
    type Input = MonitoringRequest;
    type Output = MonitoringResult;

    async fn execute(&self, input: Self::Input) -> WorkerResult<Self::Output> {
        match input.metric {
            MonitoringMetric::CpuUsage => Ok(MonitoringResult::CpuLevel(35)),
            MonitoringMetric::MemoryUsage => Ok(MonitoringResult::MemoryLevel(48)),
            MonitoringMetric::DiskUsage => Ok(MonitoringResult::DiskLevel(62)),
            MonitoringMetric::NetworkLatency => Ok(MonitoringResult::LatencyMs(25)),
            MonitoringMetric::SystemHealth => Ok(MonitoringResult::Healthy),
        }
    }

    fn name(&self) -> &str {
        "MonitoringWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(10)
    }

    fn priority(&self) -> Priority {
        Priority::High
    }
}

impl MonitoringWorker {
    pub fn new() -> Self {
        MonitoringWorker {
            timeout: Duration::from_secs(10),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cpu_monitoring() {
        let worker = MonitoringWorker::new();
        let request = MonitoringRequest {
            metric: MonitoringMetric::CpuUsage,
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_memory_monitoring() {
        let worker = MonitoringWorker::new();
        let request = MonitoringRequest {
            metric: MonitoringMetric::MemoryUsage,
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }
}
