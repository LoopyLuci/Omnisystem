/// Operational Monitoring
/// Production monitoring and alerting

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationalAlert {
    pub id: String,
    pub severity: AlertSeverity,
    pub message: String,
    pub timestamp: String,
    pub component: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationalMetrics {
    pub qps: f64,
    pub p99_latency_ms: f64,
    pub error_rate: f64,
    pub cache_hit_rate: f64,
    pub memory_usage_mb: u64,
    pub cpu_usage_percent: f64,
}

pub struct OperationalMonitoring {
    alerts: Vec<OperationalAlert>,
    metrics_history: Vec<OperationalMetrics>,
}

impl OperationalMonitoring {
    pub fn new() -> Self {
        OperationalMonitoring {
            alerts: Vec::new(),
            metrics_history: Vec::new(),
        }
    }

    pub fn record_alert(&mut self, alert: OperationalAlert) {
        tracing::warn!("Alert [{:?}]: {}", alert.severity, alert.message);
        self.alerts.push(alert);
    }

    pub fn record_metrics(&mut self, metrics: OperationalMetrics) {
        self.metrics_history.push(metrics);
    }

    pub fn check_sla_compliance(&self) -> bool {
        // SLA: 99% uptime, <100ms p99 latency, <5% error rate
        for metrics in &self.metrics_history {
            if metrics.p99_latency_ms > 100.0 {
                return false;
            }
            if metrics.error_rate > 0.05 {
                return false;
            }
        }
        true
    }

    pub fn get_alerts(&self) -> &[OperationalAlert] {
        &self.alerts
    }

    pub fn get_critical_alerts(&self) -> Vec<&OperationalAlert> {
        self.alerts
            .iter()
            .filter(|a| a.severity == AlertSeverity::Critical)
            .collect()
    }

    pub fn get_latest_metrics(&self) -> Option<&OperationalMetrics> {
        self.metrics_history.last()
    }
}

impl Default for OperationalMonitoring {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monitoring_creation() {
        let monitoring = OperationalMonitoring::new();
        assert_eq!(monitoring.get_alerts().len(), 0);
    }

    #[test]
    fn test_record_alert() {
        let mut monitoring = OperationalMonitoring::new();
        let alert = OperationalAlert {
            id: "alert1".to_string(),
            severity: AlertSeverity::Warning,
            message: "High latency detected".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            component: "resolver".to_string(),
        };
        monitoring.record_alert(alert);
        assert_eq!(monitoring.get_alerts().len(), 1);
    }

    #[test]
    fn test_sla_compliance() {
        let mut monitoring = OperationalMonitoring::new();
        let metrics = OperationalMetrics {
            qps: 1000.0,
            p99_latency_ms: 50.0,
            error_rate: 0.01,
            cache_hit_rate: 0.85,
            memory_usage_mb: 256,
            cpu_usage_percent: 45.0,
        };
        monitoring.record_metrics(metrics);
        assert!(monitoring.check_sla_compliance());
    }
}
