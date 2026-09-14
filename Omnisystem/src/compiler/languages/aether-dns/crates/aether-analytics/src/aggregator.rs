/// Metrics Aggregator
/// Time-series and categorical aggregation

use crate::metrics::{QueryMetrics, AggregatedMetrics};
use dashmap::DashMap;
use std::sync::Arc;

pub struct MetricsAggregator {
    queries: Arc<DashMap<String, QueryMetrics>>,
    domain_counts: Arc<DashMap<String, u64>>,
    threat_counts: Arc<DashMap<String, u64>>,
    source_ip_counts: Arc<DashMap<String, u64>>,
}

impl MetricsAggregator {
    pub fn new() -> Self {
        MetricsAggregator {
            queries: Arc::new(DashMap::new()),
            domain_counts: Arc::new(DashMap::new()),
            threat_counts: Arc::new(DashMap::new()),
            source_ip_counts: Arc::new(DashMap::new()),
        }
    }

    pub fn record_query(&self, metrics: QueryMetrics) {
        // Update domain counts
        self.domain_counts
            .entry(metrics.domain.clone())
            .and_modify(|c| *c += 1)
            .or_insert(1);

        // Update threat counts
        self.threat_counts
            .entry(metrics.threat_level.clone())
            .and_modify(|c| *c += 1)
            .or_insert(1);

        // Update source IP counts
        self.source_ip_counts
            .entry(metrics.source_ip.clone())
            .and_modify(|c| *c += 1)
            .or_insert(1);

        // Store full metrics
        self.queries.insert(metrics.query_id.clone(), metrics);
    }

    pub fn get_domain_stats(&self) -> Vec<(String, u64)> {
        let mut stats: Vec<_> = self
            .domain_counts
            .iter()
            .map(|entry| (entry.key().clone(), *entry.value()))
            .collect();
        stats.sort_by(|a, b| b.1.cmp(&a.1));
        stats
    }

    pub fn get_top_domains(&self, limit: usize) -> Vec<(String, u64)> {
        self.get_domain_stats().into_iter().take(limit).collect()
    }

    pub fn get_threat_distribution(&self) -> Vec<(String, u64)> {
        let mut dist: Vec<_> = self
            .threat_counts
            .iter()
            .map(|entry| (entry.key().clone(), *entry.value()))
            .collect();
        dist.sort_by(|a, b| b.1.cmp(&a.1));
        dist
    }

    pub fn get_top_sources(&self, limit: usize) -> Vec<(String, u64)> {
        let mut stats: Vec<_> = self
            .source_ip_counts
            .iter()
            .map(|entry| (entry.key().clone(), *entry.value()))
            .collect();
        stats.sort_by(|a, b| b.1.cmp(&a.1));
        stats.into_iter().take(limit).collect()
    }

    pub fn get_aggregated_metrics(&self) -> AggregatedMetrics {
        let metrics: Vec<_> = self
            .queries
            .iter()
            .map(|entry| entry.value().clone())
            .collect();
        AggregatedMetrics::from_samples(&metrics)
    }

    pub fn query_count(&self) -> usize {
        self.queries.len()
    }

    pub fn domain_count(&self) -> usize {
        self.domain_counts.len()
    }

    pub fn clear(&self) {
        self.queries.clear();
        self.domain_counts.clear();
        self.threat_counts.clear();
        self.source_ip_counts.clear();
    }
}

impl Default for MetricsAggregator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aggregator_creation() {
        let agg = MetricsAggregator::new();
        assert_eq!(agg.query_count(), 0);
    }

    #[test]
    fn test_record_query() {
        let agg = MetricsAggregator::new();
        let metrics = QueryMetrics {
            query_id: "q1".to_string(),
            domain: "example.com".to_string(),
            source_ip: "192.168.1.1".to_string(),
            query_type: "A".to_string(),
            response_code: "NOERROR".to_string(),
            latency_ms: 50,
            timestamp: "2026-06-11T00:00:00Z".to_string(),
            cached: false,
            threat_level: "None".to_string(),
            anonymity_level: 0,
            bytes_sent: 100,
            bytes_received: 200,
        };
        agg.record_query(metrics);
        assert_eq!(agg.query_count(), 1);
    }

    #[test]
    fn test_top_domains() {
        let agg = MetricsAggregator::new();
        for i in 0..5 {
            agg.record_query(QueryMetrics {
                query_id: format!("q{}", i),
                domain: "example.com".to_string(),
                source_ip: "192.168.1.1".to_string(),
                query_type: "A".to_string(),
                response_code: "NOERROR".to_string(),
                latency_ms: 50,
                timestamp: "2026-06-11T00:00:00Z".to_string(),
                cached: false,
                threat_level: "None".to_string(),
                anonymity_level: 0,
                bytes_sent: 100,
                bytes_received: 200,
            });
        }
        let top = agg.get_top_domains(1);
        assert_eq!(top.len(), 1);
        assert_eq!(top[0].1, 5);
    }
}
