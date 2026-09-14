/// Query Metrics
/// Individual query metrics and aggregations

use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryMetrics {
    pub query_id: String,
    pub domain: String,
    pub source_ip: String,
    pub query_type: String,
    pub response_code: String,
    pub latency_ms: u64,
    pub timestamp: String,
    pub cached: bool,
    pub threat_level: String,
    pub anonymity_level: u8,
    pub bytes_sent: u64,
    pub bytes_received: u64,
}

#[derive(Debug, Clone)]
pub struct AggregatedMetrics {
    pub total_queries: u64,
    pub total_cached_queries: u64,
    pub total_threats: u64,
    pub avg_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub cache_hit_rate: f64,
    pub threat_detection_rate: f64,
}

impl AggregatedMetrics {
    pub fn new() -> Self {
        AggregatedMetrics {
            total_queries: 0,
            total_cached_queries: 0,
            total_threats: 0,
            avg_latency_ms: 0.0,
            p95_latency_ms: 0.0,
            p99_latency_ms: 0.0,
            cache_hit_rate: 0.0,
            threat_detection_rate: 0.0,
        }
    }

    pub fn from_samples(metrics: &[QueryMetrics]) -> Self {
        if metrics.is_empty() {
            return AggregatedMetrics::new();
        }

        let total_queries = metrics.len() as u64;
        let cached_count = metrics.iter().filter(|m| m.cached).count() as u64;
        let threat_count = metrics
            .iter()
            .filter(|m| m.threat_level != "None")
            .count() as u64;

        let latencies: Vec<u64> = metrics.iter().map(|m| m.latency_ms).collect();
        let avg_latency = latencies.iter().sum::<u64>() as f64 / latencies.len() as f64;

        // Simple percentile approximation
        let mut sorted_latencies = latencies.clone();
        sorted_latencies.sort_unstable();
        let p95_idx = (sorted_latencies.len() as f64 * 0.95) as usize;
        let p99_idx = (sorted_latencies.len() as f64 * 0.99) as usize;

        AggregatedMetrics {
            total_queries,
            total_cached_queries: cached_count,
            total_threats: threat_count,
            avg_latency_ms: avg_latency,
            p95_latency_ms: sorted_latencies
                .get(p95_idx)
                .copied()
                .unwrap_or(0) as f64,
            p99_latency_ms: sorted_latencies
                .get(p99_idx)
                .copied()
                .unwrap_or(0) as f64,
            cache_hit_rate: cached_count as f64 / total_queries as f64,
            threat_detection_rate: threat_count as f64 / total_queries as f64,
        }
    }
}

impl Default for AggregatedMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_metrics_creation() {
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
        assert_eq!(metrics.domain, "example.com");
    }

    #[test]
    fn test_aggregated_metrics_from_samples() {
        let metrics = vec![
            QueryMetrics {
                query_id: "q1".to_string(),
                domain: "example.com".to_string(),
                source_ip: "192.168.1.1".to_string(),
                query_type: "A".to_string(),
                response_code: "NOERROR".to_string(),
                latency_ms: 50,
                timestamp: "2026-06-11T00:00:00Z".to_string(),
                cached: true,
                threat_level: "None".to_string(),
                anonymity_level: 0,
                bytes_sent: 100,
                bytes_received: 200,
            },
            QueryMetrics {
                query_id: "q2".to_string(),
                domain: "google.com".to_string(),
                source_ip: "192.168.1.2".to_string(),
                query_type: "AAAA".to_string(),
                response_code: "NOERROR".to_string(),
                latency_ms: 75,
                timestamp: "2026-06-11T00:00:01Z".to_string(),
                cached: false,
                threat_level: "None".to_string(),
                anonymity_level: 1,
                bytes_sent: 100,
                bytes_received: 200,
            },
        ];

        let agg = AggregatedMetrics::from_samples(&metrics);
        assert_eq!(agg.total_queries, 2);
        assert_eq!(agg.total_cached_queries, 1);
        assert!(agg.avg_latency_ms > 50.0 && agg.avg_latency_ms < 75.0);
    }
}
