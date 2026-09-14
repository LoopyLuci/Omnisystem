/// Health Monitoring System

use crate::worker::HealthStatus;
use dashmap::DashMap;
use std::sync::Arc;
use std::time::Instant;

pub struct HealthMonitor {
    worker_health: Arc<DashMap<String, HealthCheckData>>,
}

#[derive(Clone, Debug)]
struct HealthCheckData {
    status: HealthStatus,
    last_check: Instant,
    consecutive_failures: u32,
    recovery_start: Option<Instant>,
}

impl HealthMonitor {
    pub fn new() -> Self {
        HealthMonitor {
            worker_health: Arc::new(DashMap::new()),
        }
    }

    pub fn register_worker(&self, worker_id: String) {
        self.worker_health.insert(
            worker_id,
            HealthCheckData {
                status: HealthStatus::Healthy,
                last_check: Instant::now(),
                consecutive_failures: 0,
                recovery_start: None,
            },
        );
    }

    pub fn record_success(&self, worker_id: &str) {
        if let Some(mut data) = self.worker_health.get_mut(worker_id) {
            data.status = HealthStatus::Healthy;
            data.consecutive_failures = 0;
            data.recovery_start = None;
            data.last_check = Instant::now();
        }
    }

    pub fn record_failure(&self, worker_id: &str) {
        if let Some(mut data) = self.worker_health.get_mut(worker_id) {
            data.consecutive_failures += 1;
            data.last_check = Instant::now();

            if data.consecutive_failures >= 5 {
                data.status = HealthStatus::Unhealthy;
            } else if data.consecutive_failures >= 2 {
                data.status = HealthStatus::Degraded;
            }
        }
    }

    pub fn get_status(&self, worker_id: &str) -> Option<HealthStatus> {
        self.worker_health.get(worker_id).map(|d| d.status)
    }

    pub fn unhealthy_workers(&self) -> Vec<String> {
        self.worker_health
            .iter()
            .filter(|d| d.status == HealthStatus::Unhealthy)
            .map(|d| d.key().clone())
            .collect()
    }

    pub fn degraded_workers(&self) -> Vec<String> {
        self.worker_health
            .iter()
            .filter(|d| d.status == HealthStatus::Degraded)
            .map(|d| d.key().clone())
            .collect()
    }
}

impl Default for HealthMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_monitor() {
        let monitor = HealthMonitor::new();
        monitor.register_worker("worker1".to_string());

        monitor.record_success("worker1");
        assert_eq!(
            monitor.get_status("worker1"),
            Some(HealthStatus::Healthy)
        );

        for _ in 0..5 {
            monitor.record_failure("worker1");
        }
        assert_eq!(
            monitor.get_status("worker1"),
            Some(HealthStatus::Unhealthy)
        );
    }
}
