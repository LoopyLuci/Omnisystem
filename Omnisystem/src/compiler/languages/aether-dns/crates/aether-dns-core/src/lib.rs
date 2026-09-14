/// AETHER DNS Core - Complete DNS Protocol Implementation
/// RFC 1035: Domain Names - Implementation and Specification

pub mod protocol;
pub mod cache;
pub mod dnssec;
pub mod error;
pub mod query;
pub mod serialization;

pub use protocol::{
    DNSMessage, DNSQuestion, DNSRecord, DNSFlags, RecordType, QueryClass,
    ResourceRecord, DomainName, DNSHeader, ResponseCode,
};
pub use cache::DNSCache;
pub use dnssec::DNSSECValidator;
pub use error::{DnsError, Result};
pub use query::{DNSQuery, QueryResponse, QuerySource};
pub use serialization::{DNSSerializer, DNSDeserializer};

/// AETHER DNS Core Library
/// Provides complete DNS protocol stack with:
/// - RFC 1035 message parsing/serialization
/// - DNSSEC validation
/// - Efficient caching
/// - Query processing pipeline
pub mod prelude {
    pub use crate::{
        DNSMessage, DNSQuestion, DNSRecord, DNSFlags, RecordType,
        DNSCache, DNSSECValidator, DnsError, DNSQuery, QueryResponse,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dns_core_library() {
        // Verify library loads correctly
        let _cache = DNSCache::new(1024 * 1024 * 100); // 100MB cache
        assert!(true);
    }
}
