/// Rate Limiter
/// Prevent DNS amplification and flood attacks

use crate::threat_types::{ThreatIndicator, ThreatType};
use dashmap::DashMap;
use std::net::IpAddr;
use std::time::Instant;

pub struct RateLimiter {
    client_rates: DashMap<String, ClientRateInfo>,
    max_queries_per_minute: u32,
    max_queries_per_second: u32,
}

struct ClientRateInfo {
    queries_this_second: u32,
    queries_this_minute: u32,
    last_query: Instant,
    second_reset_time: Instant,
    minute_reset_time: Instant,
    blocked_count: u32,
}

impl RateLimiter {
    pub fn new() -> Self {
        RateLimiter {
            client_rates: DashMap::new(),
            max_queries_per_minute: 6000,  // 100 qps average
            max_queries_per_second: 500,   // Burst limit
        }
    }

    pub fn check_rate(&self, source_ip: &str) -> Option<ThreatIndicator> {
        let now = Instant::now();
        let mut entry = self.client_rates.entry(source_ip.to_string()).or_insert(
            ClientRateInfo {
                queries_this_second: 0,
                queries_this_minute: 0,
                last_query: now,
                second_reset_time: now,
                minute_reset_time: now,
                blocked_count: 0,
            },
        );

        let info = entry.value_mut();

        // Reset counters if windows have passed
        if now.duration_since(info.second_reset_time).as_secs() >= 1 {
            info.queries_this_second = 0;
            info.second_reset_time = now;
        }

        if now.duration_since(info.minute_reset_time).as_secs() >= 60 {
            info.queries_this_minute = 0;
            info.minute_reset_time = now;
        }

        // Increment counters
        info.queries_this_second += 1;
        info.queries_this_minute += 1;
        info.last_query = now;

        // Check limits
        if info.queries_this_second > self.max_queries_per_second {
            info.blocked_count += 1;
            return Some(ThreatIndicator {
                threat_type: ThreatType::RateLimitAbuse,
                confidence: 0.9,
                evidence: format!(
                    "Rate limit exceeded: {} qps (limit: {} qps)",
                    info.queries_this_second, self.max_queries_per_second
                ),
                detected_at: chrono::Utc::now().to_rfc3339(),
            });
        }

        if info.queries_this_minute > self.max_queries_per_minute {
            info.blocked_count += 1;
            return Some(ThreatIndicator {
                threat_type: ThreatType::RateLimitAbuse,
                confidence: 0.7,
                evidence: format!(
                    "Minute rate limit: {} qpm (limit: {} qpm)",
                    info.queries_this_minute, self.max_queries_per_minute
                ),
                detected_at: chrono::Utc::now().to_rfc3339(),
            });
        }

        None
    }

    pub fn set_limits(&mut self, per_second: u32, per_minute: u32) {
        self.max_queries_per_second = per_second;
        self.max_queries_per_minute = per_minute;
    }

    pub fn get_client_rate(&self, source_ip: &str) -> Option<(u32, u32)> {
        self.client_rates
            .get(source_ip)
            .map(|info| (info.queries_this_second, info.queries_this_minute))
    }

    pub fn get_limited_ips(&self) -> usize {
        self.client_rates
            .iter()
            .filter(|entry| entry.value().blocked_count > 0)
            .count()
    }

    pub fn clear_limits(&self) {
        self.client_rates.clear();
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_creation() {
        let limiter = RateLimiter::new();
        assert_eq!(limiter.max_queries_per_second, 500);
        assert_eq!(limiter.max_queries_per_minute, 6000);
    }

    #[test]
    fn test_check_rate_normal() {
        let limiter = RateLimiter::new();
        let result = limiter.check_rate("192.168.1.1");
        assert!(result.is_none());
    }

    #[test]
    fn test_set_limits() {
        let mut limiter = RateLimiter::new();
        limiter.set_limits(100, 1000);
        assert_eq!(limiter.max_queries_per_second, 100);
        assert_eq!(limiter.max_queries_per_minute, 1000);
    }

    #[test]
    fn test_get_client_rate() {
        let limiter = RateLimiter::new();
        limiter.check_rate("192.168.1.1");
        let rate = limiter.get_client_rate("192.168.1.1");
        assert!(rate.is_some());
    }
}
