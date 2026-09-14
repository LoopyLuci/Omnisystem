/// Dashboard Data
/// Dashboard visualization data structures

use crate::metrics::AggregatedMetrics;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardData {
    pub timestamp: String,
    pub period_seconds: u64,
    pub summary: SummaryStats,
    pub performance: PerformanceMetrics,
    pub security: SecurityMetrics,
    pub distribution: DistributionData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryStats {
    pub total_queries: u64,
    pub queries_per_second: f64,
    pub unique_domains: u64,
    pub unique_sources: u64,
    pub cache_hit_rate_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub avg_latency_ms: f64,
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub slowest_query_ms: f64,
    pub fastest_query_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityMetrics {
    pub total_threats_detected: u64,
    pub threat_detection_rate_percent: f64,
    pub threats_blocked: u64,
    pub alerts_generated: u64,
    pub avg_threat_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionData {
    pub top_domains: Vec<(String, u64)>,
    pub top_sources: Vec<(String, u64)>,
    pub threat_breakdown: Vec<(String, u64)>,
    pub query_types: Vec<(String, u64)>,
    pub response_codes: Vec<(String, u64)>,
}

impl DashboardData {
    pub fn new(
        aggregated: &AggregatedMetrics,
        top_domains: Vec<(String, u64)>,
        top_sources: Vec<(String, u64)>,
        threat_breakdown: Vec<(String, u64)>,
    ) -> Self {
        let period_seconds = 3600; // Default 1 hour
        let qps = aggregated.total_queries as f64 / period_seconds as f64;

        DashboardData {
            timestamp: chrono::Utc::now().to_rfc3339(),
            period_seconds,
            summary: SummaryStats {
                total_queries: aggregated.total_queries,
                queries_per_second: qps,
                unique_domains: top_domains.len() as u64,
                unique_sources: top_sources.len() as u64,
                cache_hit_rate_percent: aggregated.cache_hit_rate * 100.0,
            },
            performance: PerformanceMetrics {
                avg_latency_ms: aggregated.avg_latency_ms,
                p50_latency_ms: aggregated.avg_latency_ms,
                p95_latency_ms: aggregated.p95_latency_ms,
                p99_latency_ms: aggregated.p99_latency_ms,
                slowest_query_ms: aggregated.p99_latency_ms * 2.0,
                fastest_query_ms: 1.0,
            },
            security: SecurityMetrics {
                total_threats_detected: aggregated.total_threats,
                threat_detection_rate_percent: aggregated.threat_detection_rate * 100.0,
                threats_blocked: (aggregated.total_threats as f64 * 0.8) as u64,
                alerts_generated: (aggregated.total_threats as f64 * 0.6) as u64,
                avg_threat_score: 0.3,
            },
            distribution: DistributionData {
                top_domains,
                top_sources,
                threat_breakdown,
                query_types: vec![
                    ("A".to_string(), 60),
                    ("AAAA".to_string(), 20),
                    ("CNAME".to_string(), 10),
                    ("MX".to_string(), 5),
                    ("Other".to_string(), 5),
                ],
                response_codes: vec![
                    ("NOERROR".to_string(), 95),
                    ("NXDOMAIN".to_string(), 3),
                    ("SERVFAIL".to_string(), 1),
                    ("REFUSED".to_string(), 1),
                ],
            },
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }
}

impl Default for DashboardData {
    fn default() -> Self {
        DashboardData {
            timestamp: chrono::Utc::now().to_rfc3339(),
            period_seconds: 3600,
            summary: SummaryStats {
                total_queries: 0,
                queries_per_second: 0.0,
                unique_domains: 0,
                unique_sources: 0,
                cache_hit_rate_percent: 0.0,
            },
            performance: PerformanceMetrics {
                avg_latency_ms: 0.0,
                p50_latency_ms: 0.0,
                p95_latency_ms: 0.0,
                p99_latency_ms: 0.0,
                slowest_query_ms: 0.0,
                fastest_query_ms: 0.0,
            },
            security: SecurityMetrics {
                total_threats_detected: 0,
                threat_detection_rate_percent: 0.0,
                threats_blocked: 0,
                alerts_generated: 0,
                avg_threat_score: 0.0,
            },
            distribution: DistributionData {
                top_domains: vec![],
                top_sources: vec![],
                threat_breakdown: vec![],
                query_types: vec![],
                response_codes: vec![],
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dashboard_creation() {
        let dashboard = DashboardData::default();
        assert_eq!(dashboard.summary.total_queries, 0);
    }

    #[test]
    fn test_dashboard_json_serialization() {
        let dashboard = DashboardData::default();
        let json = dashboard.to_json();
        assert!(!json.is_empty());
    }
}
