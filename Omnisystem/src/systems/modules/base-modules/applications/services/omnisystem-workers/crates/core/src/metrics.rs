/// Metrics Collection

use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

pub struct MetricsCollector {
    counters: Arc<DashMap<String, Arc<AtomicU64>>>,
    gauges: Arc<DashMap<String, f64>>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        MetricsCollector {
            counters: Arc::new(DashMap::new()),
            gauges: Arc::new(DashMap::new()),
        }
    }

    pub fn increment_counter(&self, name: &str, amount: u64) {
        let counter = self.counters
            .entry(name.to_string())
            .or_insert_with(|| Arc::new(AtomicU64::new(0)))
            .clone();
        counter.fetch_add(amount, Ordering::Relaxed);
    }

    pub fn set_gauge(&self, name: &str, value: f64) {
        self.gauges.insert(name.to_string(), value);
    }

    pub fn get_counter(&self, name: &str) -> Option<u64> {
        self.counters.get(name).map(|c| c.load(Ordering::Relaxed))
    }

    pub fn get_gauge(&self, name: &str) -> Option<f64> {
        self.gauges.get(name).map(|g| *g.value())
    }

    pub fn all_metrics(&self) -> MetricsSnapshot {
        let counters = self.counters
            .iter()
            .map(|e| (e.key().clone(), e.value().load(Ordering::Relaxed)))
            .collect();

        let gauges = self.gauges
            .iter()
            .map(|e| (e.key().clone(), *e.value()))
            .collect();

        MetricsSnapshot { counters, gauges }
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub counters: std::collections::HashMap<String, u64>,
    pub gauges: std::collections::HashMap<String, f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_collector() {
        let collector = MetricsCollector::new();
        collector.increment_counter("requests", 1);
        collector.set_gauge("cpu_usage", 0.75);

        assert_eq!(collector.get_counter("requests"), Some(1));
        assert_eq!(collector.get_gauge("cpu_usage"), Some(0.75));
    }
}
