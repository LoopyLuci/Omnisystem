/// Main Query Processor

use aether_dns_core::cache::DNSCache;
use aether_dns_core::query::{DNSQuery, QueryResponse};
use aether_dns_core::protocol::DNSMessage;
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, info};
use uuid::Uuid;
use crate::pipeline::ProcessingContext;
use crate::upstream::UpstreamResolver;
use crate::response_builder::ResponseBuilder;

pub struct QueryProcessor {
    cache: Arc<DNSCache>,
    upstream_resolver: Arc<UpstreamResolver>,
    response_builder: Arc<ResponseBuilder>,
    request_timeout_ms: u64,
}

impl QueryProcessor {
    pub fn new(
        cache: Arc<DNSCache>,
        upstream_resolver: Arc<UpstreamResolver>,
    ) -> Self {
        QueryProcessor {
            cache,
            upstream_resolver,
            response_builder: Arc::new(ResponseBuilder::new()),
            request_timeout_ms: 5000,
        }
    }

    pub async fn process(&self, query: DNSQuery) -> anyhow::Result<QueryResponse> {
        let start = Instant::now();
        let query_id = Uuid::new_v4();

        debug!("Processing query: {} {}", query.domain, query.query_type);

        // Stage 1: Validation
        self.validate_query(&query)?;

        // Stage 2: Cache lookup
        if let Some(cached) = self.cache.get(&query.domain).await {
            let latency = start.elapsed().as_millis() as u32;
            info!("Cache hit for {}: {}ms", query.domain, latency);
            return Ok(QueryResponse::from_cache(query_id, cached, latency));
        }

        // Stage 3: Upstream resolution
        let response = self.upstream_resolver.resolve(&query).await?;

        // Stage 4: Response building
        let final_response = self.response_builder.build(&query, response)?;

        // Stage 5: Cache store
        self.cache.set(
            query.domain.clone(),
            final_response.answers.clone(),
            final_response.ttl,
        ).await;

        let latency = start.elapsed().as_millis() as u32;
        info!("Query processed in {}ms", latency);

        Ok(QueryResponse {
            query_id,
            status_code: 0,
            answers: final_response.answers,
            authorities: final_response.authorities,
            additionals: final_response.additionals,
            ttl: final_response.ttl,
            served_from: "upstream".to_string(),
            latency_ms: latency,
            cached: false,
        })
    }

    fn validate_query(&self, query: &DNSQuery) -> anyhow::Result<()> {
        // Validate domain name
        if query.domain.is_empty() {
            return Err(anyhow::anyhow!("Empty domain name"));
        }

        if query.domain.len() > 255 {
            return Err(anyhow::anyhow!("Domain name too long"));
        }

        // Validate source IP
        if query.source_ip.is_empty() {
            return Err(anyhow::anyhow!("Missing source IP"));
        }

        Ok(())
    }

    pub fn set_timeout(&mut self, timeout_ms: u64) {
        self.request_timeout_ms = timeout_ms;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_processor_creation() {
        let cache = Arc::new(DNSCache::new(1024 * 1024));
        let resolver = Arc::new(UpstreamResolver::new());
        let processor = QueryProcessor::new(cache, resolver);
        assert_eq!(processor.request_timeout_ms, 5000);
    }

    #[test]
    fn test_query_validation() {
        let cache = Arc::new(DNSCache::new(1024 * 1024));
        let resolver = Arc::new(UpstreamResolver::new());
        let processor = QueryProcessor::new(cache, resolver);

        let query = DNSQuery::new(
            "example.com".to_string(),
            aether_dns_core::protocol::RecordType::A,
            aether_dns_core::query::QuerySource::UDP,
            "127.0.0.1".to_string(),
        );

        assert!(processor.validate_query(&query).is_ok());
    }
}
