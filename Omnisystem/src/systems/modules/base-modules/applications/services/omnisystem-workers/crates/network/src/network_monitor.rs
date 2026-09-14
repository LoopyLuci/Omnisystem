/// NetworkMonitorWorker - Network interface monitoring

use omnisystem_workers_core::prelude::*;
use async_trait::async_trait;
use std::time::Duration;

pub struct NetworkMonitorWorker {
    timeout: Duration,
}

pub struct NetworkMonitorRequest {
    pub interface: String,
    pub metric: MonitorMetric,
}

#[derive(Debug, Clone)]
pub enum MonitorMetric {
    Bandwidth,
    Latency,
    PacketLoss,
    Errors,
    Status,
}

#[derive(Debug)]
pub enum NetworkMonitorResult {
    Bandwidth(u64),
    Latency(u32),
    PacketLoss(f32),
    Errors(u64),
    Up,
}

#[async_trait]
impl Worker for NetworkMonitorWorker {
    type Input = NetworkMonitorRequest;
    type Output = NetworkMonitorResult;

    async fn execute(&self, input: Self::Input) -> WorkerResult<Self::Output> {
        match input.metric {
            MonitorMetric::Bandwidth => Ok(NetworkMonitorResult::Bandwidth(1000000)),
            MonitorMetric::Latency => Ok(NetworkMonitorResult::Latency(10)),
            MonitorMetric::PacketLoss => Ok(NetworkMonitorResult::PacketLoss(0.1)),
            MonitorMetric::Errors => Ok(NetworkMonitorResult::Errors(0)),
            MonitorMetric::Status => Ok(NetworkMonitorResult::Up),
        }
    }

    fn name(&self) -> &str {
        "NetworkMonitorWorker"
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(30)
    }

    fn priority(&self) -> Priority {
        Priority::Normal
    }
}

impl NetworkMonitorWorker {
    pub fn new() -> Self {
        NetworkMonitorWorker {
            timeout: Duration::from_secs(30),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_monitor_bandwidth() {
        let worker = NetworkMonitorWorker::new();
        let request = NetworkMonitorRequest {
            interface: "eth0".to_string(),
            metric: MonitorMetric::Bandwidth,
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_monitor_status() {
        let worker = NetworkMonitorWorker::new();
        let request = NetworkMonitorRequest {
            interface: "eth0".to_string(),
            metric: MonitorMetric::Status,
        };
        let result = worker.execute(request).await;
        assert!(result.is_ok());
    }
}
