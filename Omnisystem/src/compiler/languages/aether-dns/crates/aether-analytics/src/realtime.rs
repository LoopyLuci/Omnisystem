/// Realtime Monitoring
/// Real-time metrics streaming and monitoring

use crate::metrics::QueryMetrics;
use dashmap::DashMap;
use std::sync::Arc;

pub struct RealtimeMonitor {
    current_window: Arc<DashMap<String, u64>>,
    window_duration_secs: u64,
}

impl RealtimeMonitor {
    pub fn new(window_duration_secs: u64) -> Self {
        RealtimeMonitor {
            current_window: Arc::new(DashMap::new()),
            window_duration_secs,
        }
    }

    pub fn record_metric(&self, metric: &QueryMetrics) {
        let key = format!("queries:{}:minute", chrono::Utc::now().timestamp() / 60);
        self.current_window
            .entry(key)
            .and_modify(|v| *v += 1)
            .or_insert(1);
    }

    pub fn get_qps(&self) -> f64 {
        let total: u64 = self.current_window.iter().map(|e| *e.value()).sum();
        total as f64 / self.window_duration_secs as f64
    }

    pub fn get_queries_in_window(&self) -> u64 {
        self.current_window.iter().map(|e| *e.value()).sum()
    }

    pub fn reset_window(&self) {
        self.current_window.clear();
    }

    pub async fn monitor_continuous(
        &self,
        interval_secs: u64,
    ) {
        tokio::spawn({
            let window = Arc::clone(&self.current_window);
            async move {
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_secs(interval_secs))
                        .await;
                    window.clear();
                }
            }
        });
    }
}

impl Default for RealtimeMonitor {
    fn default() -> Self {
        Self::new(60)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_realtime_monitor_creation() {
        let monitor = RealtimeMonitor::new(60);
        assert_eq!(monitor.window_duration_secs, 60);
    }

    #[test]
    fn test_record_metric() {
        let monitor = RealtimeMonitor::new(60);
        let metric = QueryMetrics {
            query_id: "q1".to_string(),
            domain: "example.com".to_string(),
            source_ip: "192.168.1.1".to_string(),
            query_type: "A".to_string(),
            response_code: "NOERROR".to_string(),
            latency_ms: 50,
            timestamp: chrono::Utc::now().to_rfc3339(),
            cached: false,
            threat_level: "None".to_string(),
            anonymity_level: 0,
            bytes_sent: 100,
            bytes_received: 200,
        };
        monitor.record_metric(&metric);
        assert!(monitor.get_queries_in_window() > 0);
    }

    #[test]
    fn test_qps_calculation() {
        let monitor = RealtimeMonitor::new(60);
        let metric = QueryMetrics {
            query_id: "q1".to_string(),
            domain: "example.com".to_string(),
            source_ip: "192.168.1.1".to_string(),
            query_type: "A".to_string(),
            response_code: "NOERROR".to_string(),
            latency_ms: 50,
            timestamp: chrono::Utc::now().to_rfc3339(),
            cached: false,
            threat_level: "None".to_string(),
            anonymity_level: 0,
            bytes_sent: 100,
            bytes_received: 200,
        };
        monitor.record_metric(&metric);
        let qps = monitor.get_qps();
        assert!(qps > 0.0);
    }
}
