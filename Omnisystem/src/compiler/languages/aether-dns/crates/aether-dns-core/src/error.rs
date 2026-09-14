/// Error types for DNS operations

use thiserror::Error;

#[derive(Error, Debug)]
pub enum DnsError {
    #[error("DNS format error: {0}")]
    FormatError(String),

    #[error("Server failure: {0}")]
    ServerError(String),

    #[error("Domain not found: {0}")]
    NxDomain(String),

    #[error("Query not implemented: {0}")]
    NotImplemented(String),

    #[error("Query refused: {0}")]
    Refused(String),

    #[error("Cache error: {0}")]
    CacheError(String),

    #[error("DNSSEC validation failed: {0}")]
    DnssecError(String),

    #[error("Invalid record data: {0}")]
    InvalidRecord(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Unknown error: {0}")]
    Unknown(String),

    #[error("Timeout")]
    Timeout,

    #[error("Rate limited")]
    RateLimited,

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
}

pub type Result<T> = std::result::Result<T, DnsError>;

impl DnsError {
    pub fn format_error(msg: impl Into<String>) -> Self {
        DnsError::FormatError(msg.into())
    }

    pub fn server_error(msg: impl Into<String>) -> Self {
        DnsError::ServerError(msg.into())
    }

    pub fn nx_domain(domain: impl Into<String>) -> Self {
        DnsError::NxDomain(domain.into())
    }

    pub fn cache_error(msg: impl Into<String>) -> Self {
        DnsError::CacheError(msg.into())
    }

    pub fn dnssec_error(msg: impl Into<String>) -> Self {
        DnsError::DnssecError(msg.into())
    }

    pub fn parse_error(msg: impl Into<String>) -> Self {
        DnsError::ParseError(msg.into())
    }

    pub fn invalid_record(msg: impl Into<String>) -> Self {
        DnsError::InvalidRecord(msg.into())
    }

    pub fn invalid_parameter(msg: impl Into<String>) -> Self {
        DnsError::InvalidParameter(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = DnsError::format_error("invalid format");
        assert_eq!(err.to_string(), "DNS format error: invalid format");
    }

    #[test]
    fn test_error_types() {
        let _format_err = DnsError::FormatError("test".to_string());
        let _server_err = DnsError::ServerError("test".to_string());
        let _nx_err = DnsError::NxDomain("example.com".to_string());
    }
}
