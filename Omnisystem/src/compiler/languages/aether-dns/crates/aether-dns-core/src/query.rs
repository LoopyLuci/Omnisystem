/// DNS Query Processing Context

use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::protocol::{RecordType, DNSRecord};

/// Query Source Protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QuerySource {
    UDP,    // Standard DNS (port 53)
    DoH,    // DNS-over-HTTPS
    DoT,    // DNS-over-TLS
    DoQ,    // DNS-over-QUIC
}

impl QuerySource {
    pub fn to_string(&self) -> String {
        match self {
            QuerySource::UDP => "UDP".to_string(),
            QuerySource::DoH => "DoH".to_string(),
            QuerySource::DoT => "DoT".to_string(),
            QuerySource::DoQ => "DoQ".to_string(),
        }
    }
}

/// DNS Query Context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DNSQuery {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub domain: String,
    pub query_type: RecordType,
    pub source: QuerySource,
    pub source_ip: String,
    pub anonymity_level: u8,  // 0-5: higher = more anonymous
    pub user_id: Option<String>,
}

impl DNSQuery {
    pub fn new(domain: String, query_type: RecordType, source: QuerySource, source_ip: String) -> Self {
        DNSQuery {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            domain,
            query_type,
            source,
            source_ip,
            anonymity_level: 0,
            user_id: None,
        }
    }

    pub fn with_anonymity(mut self, level: u8) -> Self {
        self.anonymity_level = std::cmp::min(level, 5);
        self
    }

    pub fn with_user(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn elapsed_ms(&self) -> u128 {
        Utc::now().signed_duration_since(self.timestamp).num_milliseconds() as u128
    }
}

/// Query Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResponse {
    pub query_id: Uuid,
    pub status_code: u8,  // 0 = success, 1-5 = error codes
    pub answers: Vec<DNSRecord>,
    pub authorities: Vec<DNSRecord>,
    pub additionals: Vec<DNSRecord>,
    pub ttl: u32,
    pub served_from: String,  // "cache", "upstream", "relay"
    pub latency_ms: u32,
    pub cached: bool,
}

impl QueryResponse {
    pub fn success(query_id: Uuid, answers: Vec<DNSRecord>) -> Self {
        QueryResponse {
            query_id,
            status_code: 0,
            answers,
            authorities: vec![],
            additionals: vec![],
            ttl: 300,
            served_from: "upstream".to_string(),
            latency_ms: 0,
            cached: false,
        }
    }

    pub fn error(query_id: Uuid, code: u8) -> Self {
        QueryResponse {
            query_id,
            status_code: code,
            answers: vec![],
            authorities: vec![],
            additionals: vec![],
            ttl: 0,
            served_from: "error".to_string(),
            latency_ms: 0,
            cached: false,
        }
    }

    pub fn from_cache(query_id: Uuid, answers: Vec<DNSRecord>, latency_ms: u32) -> Self {
        QueryResponse {
            query_id,
            status_code: 0,
            answers,
            authorities: vec![],
            additionals: vec![],
            ttl: 300,
            served_from: "cache".to_string(),
            latency_ms,
            cached: true,
        }
    }

    pub fn is_success(&self) -> bool {
        self.status_code == 0
    }

    pub fn error_name(&self) -> String {
        match self.status_code {
            0 => "NoError".to_string(),
            1 => "FormErr".to_string(),
            2 => "ServFail".to_string(),
            3 => "NxDomain".to_string(),
            4 => "NotImpl".to_string(),
            5 => "Refused".to_string(),
            _ => "Unknown".to_string(),
        }
    }
}

/// Query Statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryStats {
    pub total_queries: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub avg_latency_ms: f64,
    pub error_count: u64,
    pub success_count: u64,
}

impl QueryStats {
    pub fn new() -> Self {
        QueryStats {
            total_queries: 0,
            cache_hits: 0,
            cache_misses: 0,
            avg_latency_ms: 0.0,
            error_count: 0,
            success_count: 0,
        }
    }

    pub fn record_query(&mut self, response: &QueryResponse) {
        self.total_queries += 1;

        if response.cached {
            self.cache_hits += 1;
        } else {
            self.cache_misses += 1;
        }

        if response.is_success() {
            self.success_count += 1;
        } else {
            self.error_count += 1;
        }

        // Update average latency
        self.avg_latency_ms = (self.avg_latency_ms * (self.total_queries - 1) as f64
            + response.latency_ms as f64) / self.total_queries as f64;
    }

    pub fn cache_hit_rate(&self) -> f64 {
        if self.total_queries == 0 {
            0.0
        } else {
            self.cache_hits as f64 / self.total_queries as f64
        }
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_queries == 0 {
            0.0
        } else {
            self.success_count as f64 / self.total_queries as f64
        }
    }
}

impl Default for QueryStats {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_creation() {
        let query = DNSQuery::new(
            "example.com".to_string(),
            RecordType::A,
            QuerySource::UDP,
            "192.168.1.1".to_string(),
        );
        assert_eq!(query.domain, "example.com");
        assert_eq!(query.anonymity_level, 0);
    }

    #[test]
    fn test_query_with_anonymity() {
        let query = DNSQuery::new(
            "example.com".to_string(),
            RecordType::A,
            QuerySource::DoH,
            "192.168.1.1".to_string(),
        ).with_anonymity(3);
        assert_eq!(query.anonymity_level, 3);
    }

    #[test]
    fn test_response_creation() {
        let id = Uuid::new_v4();
        let response = QueryResponse::success(id, vec![]);
        assert!(response.is_success());
        assert_eq!(response.status_code, 0);
    }

    #[test]
    fn test_query_stats() {
        let mut stats = QueryStats::new();
        let response = QueryResponse::success(Uuid::new_v4(), vec![]);
        stats.record_query(&response);
        assert_eq!(stats.total_queries, 1);
        assert_eq!(stats.success_count, 1);
    }
}
