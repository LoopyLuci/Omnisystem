/// Response Builder
/// Constructs DNS responses from upstream results

use aether_dns_core::protocol::{DNSMessage, DNSRecord};
use aether_dns_core::query::DNSQuery;

pub struct ResponseBuilder;

impl ResponseBuilder {
    pub fn new() -> Self {
        ResponseBuilder
    }

    pub fn build(
        &self,
        query: &DNSQuery,
        mut upstream_response: DNSMessage,
    ) -> anyhow::Result<ResponseMessage> {
        // Set response flags
        upstream_response.header.flags.qr = true;  // This is a response
        upstream_response.header.flags.rd = true;  // Recursion desired

        Ok(ResponseMessage {
            answers: upstream_response.answers,
            authorities: upstream_response.authorities,
            additionals: upstream_response.additionals,
            ttl: 300,  // Default 5 minute TTL
            status: 0,
        })
    }

    pub fn error_response(&self, error_code: u8) -> ResponseMessage {
        ResponseMessage {
            answers: vec![],
            authorities: vec![],
            additionals: vec![],
            ttl: 0,
            status: error_code,
        }
    }

    pub fn nxdomain_response(&self) -> ResponseMessage {
        self.error_response(3)  // NXDOMAIN
    }

    pub fn servfail_response(&self) -> ResponseMessage {
        self.error_response(2)  // SERVFAIL
    }
}

impl Default for ResponseBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ResponseMessage {
    pub answers: Vec<DNSRecord>,
    pub authorities: Vec<DNSRecord>,
    pub additionals: Vec<DNSRecord>,
    pub ttl: u32,
    pub status: u8,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_builder_creation() {
        let builder = ResponseBuilder::new();
        let error_resp = builder.error_response(5);
        assert_eq!(error_resp.status, 5);
    }

    #[test]
    fn test_nxdomain_response() {
        let builder = ResponseBuilder::new();
        let resp = builder.nxdomain_response();
        assert_eq!(resp.status, 3);
    }

    #[test]
    fn test_servfail_response() {
        let builder = ResponseBuilder::new();
        let resp = builder.servfail_response();
        assert_eq!(resp.status, 2);
    }
}
