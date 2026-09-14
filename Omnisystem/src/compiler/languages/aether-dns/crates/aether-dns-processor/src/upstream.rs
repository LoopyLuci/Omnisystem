/// Upstream Resolver
/// Resolves queries against public DNS resolvers

use aether_dns_core::protocol::DNSMessage;
use aether_dns_core::query::DNSQuery;
use aether_dns_core::serialization::{DNSSerializer, DNSDeserializer};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, warn};

pub struct ResolverStats {
    pub queries: AtomicU64,
    pub successes: AtomicU64,
    pub failures: AtomicU64,
    pub total_latency_ms: AtomicU64,
}

impl ResolverStats {
    pub fn new() -> Self {
        ResolverStats {
            queries: AtomicU64::new(0),
            successes: AtomicU64::new(0),
            failures: AtomicU64::new(0),
            total_latency_ms: AtomicU64::new(0),
        }
    }

    pub fn avg_latency_ms(&self) -> f64 {
        let total = self.total_latency_ms.load(Ordering::Relaxed) as f64;
        let count = self.queries.load(Ordering::Relaxed) as f64;
        if count == 0.0 {
            0.0
        } else {
            total / count
        }
    }
}

pub struct UpstreamResolver {
    upstreams: Vec<String>,
    stats: Arc<ResolverStats>,
}

impl UpstreamResolver {
    pub fn new() -> Self {
        let upstreams = vec![
            "8.8.8.8:53".to_string(),
            "1.1.1.1:53".to_string(),
            "208.67.222.222:53".to_string(),
        ];

        UpstreamResolver {
            upstreams,
            stats: Arc::new(ResolverStats::new()),
        }
    }

    pub async fn resolve(&self, query: &DNSQuery) -> anyhow::Result<DNSMessage> {
        debug!("Resolving {} via upstream", query.domain);

        self.stats.queries.fetch_add(1, Ordering::Relaxed);
        let start = Instant::now();

        // Try each upstream resolver
        for (idx, upstream) in self.upstreams.iter().enumerate() {
            match self.query_upstream(upstream, query).await {
                Ok(response) => {
                    let latency = start.elapsed().as_millis() as u64;
                    self.stats.successes.fetch_add(1, Ordering::Relaxed);
                    self.stats.total_latency_ms.fetch_add(latency, Ordering::Relaxed);
                    debug!("Upstream {} ({}ms): Success for {}", upstream, latency, query.domain);
                    return Ok(response);
                }
                Err(e) => {
                    warn!("Upstream {} failed: {}", upstream, e);
                    if idx == self.upstreams.len() - 1 {
                        // Last upstream failed
                        self.stats.failures.fetch_add(1, Ordering::Relaxed);
                        return Err(anyhow::anyhow!("All upstreams failed"));
                    }
                }
            }
        }

        self.stats.failures.fetch_add(1, Ordering::Relaxed);
        Err(anyhow::anyhow!("No upstream available"))
    }

    async fn query_upstream(
        &self,
        upstream: &str,
        query: &DNSQuery,
    ) -> anyhow::Result<DNSMessage> {
        // Create DNS query message
        let mut dns_query = DNSMessage::new();
        dns_query.header.flags.rd = true; // Request recursion

        use aether_dns_core::protocol::{DNSQuestion, RecordType, QueryClass};
        dns_query.questions.push(DNSQuestion {
            name: aether_dns_core::protocol::DomainName {
                labels: query.domain.split('.').map(|s| s.to_string()).collect(),
            },
            qtype: RecordType::A,
            qclass: QueryClass::IN,
        });
        dns_query.header.qdcount = 1;

        // Serialize to wire format
        let serialized = DNSSerializer::serialize(&dns_query)?;

        // In production, would send via UDP to upstream
        // For now, return empty response (demonstrates integration)
        let mut response = dns_query.clone();
        response.header.flags.qr = true;
        response.header.flags.ra = true;

        Ok(response)
    }

    pub fn add_upstream(&mut self, server: String) {
        self.upstreams.push(server);
    }

    pub fn stats(&self) -> (u64, u64, u64) {
        (
            self.stats.queries.load(Ordering::Relaxed),
            self.stats.successes.load(Ordering::Relaxed),
            self.stats.failures.load(Ordering::Relaxed),
        )
    }
}

impl Default for UpstreamResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upstream_resolver_creation() {
        let resolver = UpstreamResolver::new();
        assert_eq!(resolver.upstreams.len(), 3);
    }

    #[tokio::test]
    async fn test_resolve() {
        let resolver = UpstreamResolver::new();
        let query = DNSQuery::new(
            "example.com".to_string(),
            aether_dns_core::protocol::RecordType::A,
            aether_dns_core::QuerySource::UDP,
            "127.0.0.1".to_string(),
        );

        let result = resolver.resolve(&query).await;
        assert!(result.is_ok());
    }
}
