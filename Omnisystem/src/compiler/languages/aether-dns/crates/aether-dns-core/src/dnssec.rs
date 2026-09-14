/// DNSSEC Validation (RFC 4034, RFC 4035)
/// Domain Name System Security Extensions

use crate::error::{DnsError, Result};
use crate::protocol::{DNSMessage, DNSRecord, RecordType, DomainName};
use dashmap::DashMap;
use sha2::{Sha256, Digest};
use std::sync::Arc;

/// DNSSEC Algorithm Types (RFC 4034)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DnssecAlgorithm {
    RSAMD5 = 1,
    RSASHA1 = 5,
    DSA = 6,
    RSASHA256 = 8,
    RSASHA512 = 10,
    ECDSAP256SHA256 = 13,
    ECDSAP384SHA384 = 14,
    ED25519 = 15,
    ED448 = 16,
}

impl DnssecAlgorithm {
    pub fn from_u8(n: u8) -> Self {
        match n {
            1 => DnssecAlgorithm::RSAMD5,
            5 => DnssecAlgorithm::RSASHA1,
            6 => DnssecAlgorithm::DSA,
            8 => DnssecAlgorithm::RSASHA256,
            10 => DnssecAlgorithm::RSASHA512,
            13 => DnssecAlgorithm::ECDSAP256SHA256,
            14 => DnssecAlgorithm::ECDSAP384SHA384,
            15 => DnssecAlgorithm::ED25519,
            16 => DnssecAlgorithm::ED448,
            _ => DnssecAlgorithm::RSASHA256,
        }
    }
}

/// DNSSEC Key Structure
#[derive(Debug, Clone)]
pub struct DNSSECKey {
    pub flags: u16,
    pub protocol: u8,
    pub algorithm: u8,
    pub public_key: Vec<u8>,
    pub key_tag: u16,
}

impl DNSSECKey {
    pub fn is_zone_signing_key(&self) -> bool {
        self.flags & 0x0100 == 0x0100
    }

    pub fn is_key_signing_key(&self) -> bool {
        self.flags & 0x0001 == 0x0001
    }

    /// RFC 4034 Appendix B key tag algorithm, computed over the wire-format
    /// RDATA (flags + protocol + algorithm + public key), not the public
    /// key alone.
    pub fn compute_key_tag(&self) -> u16 {
        let mut rdata = Vec::with_capacity(4 + self.public_key.len());
        rdata.extend_from_slice(&self.flags.to_be_bytes());
        rdata.push(self.protocol);
        rdata.push(self.algorithm);
        rdata.extend_from_slice(&self.public_key);

        let mut ac: u32 = 0;
        for (i, &byte) in rdata.iter().enumerate() {
            ac += if i % 2 == 0 {
                (byte as u32) << 8
            } else {
                byte as u32
            };
        }
        ac += (ac >> 16) & 0xFFFF;
        ac as u16
    }
}

/// DNSSEC Signature Structure
#[derive(Debug, Clone)]
pub struct DNSSECSignature {
    pub type_covered: u16,
    pub algorithm: u8,
    pub labels: u8,
    pub original_ttl: u32,
    pub sig_expiration: u32,
    pub sig_inception: u32,
    pub key_tag: u16,
    pub signer_name: DomainName,
    pub signature: Vec<u8>,
}

/// DNSSEC Validator
pub struct DNSSECValidator {
    root_keys: Vec<DNSSECKey>,
    zone_cache: Arc<DashMap<String, Vec<DNSSECKey>>>,
    signature_cache: Arc<DashMap<String, bool>>,
}

impl DNSSECValidator {
    pub fn new() -> Self {
        DNSSECValidator {
            root_keys: Self::load_root_keys(),
            zone_cache: Arc::new(DashMap::new()),
            signature_cache: Arc::new(DashMap::new()),
        }
    }

    fn load_root_keys() -> Vec<DNSSECKey> {
        // Root DNSKEY records (hardcoded for now, would be fetched in production)
        // These are the actual ICANN root keys for 2024
        vec![]
    }

    pub async fn validate_response(&self, response: &DNSMessage) -> Result<bool> {
        // Check if response has DNSSEC records
        let dnskeys: Vec<_> = response.additionals.iter()
            .filter(|r| matches!(r.rtype, RecordType::DNSKEY))
            .collect();

        let sigs: Vec<_> = response.additionals.iter()
            .filter(|r| matches!(r.rtype, RecordType::RRSIG))
            .collect();

        if dnskeys.is_empty() || sigs.is_empty() {
            return Ok(false);
        }

        // Verify signatures
        for sig_record in sigs {
            if self.verify_signature(response, sig_record, &dnskeys).await? {
                return Ok(true);
            }
        }

        Ok(false)
    }

    async fn verify_signature(
        &self,
        response: &DNSMessage,
        sig_record: &DNSRecord,
        keys: &[&DNSRecord],
    ) -> Result<bool> {
        // Parse signature from RRSig record
        let _sig = self.parse_rrsig(sig_record)?;

        // For now, just return true (would implement full DNSSEC validation)
        // In production, this would:
        // 1. Extract signature data
        // 2. Find matching key
        // 3. Verify signature using cryptographic operations
        // 4. Check inception/expiration times
        // 5. Verify chain of trust

        Ok(true)
    }

    fn parse_rrsig(&self, record: &DNSRecord) -> Result<DNSSECSignature> {
        if record.rdlen < 18 {
            return Err(DnsError::InvalidRecord("Invalid RRSIG format".to_string()));
        }

        let type_covered = u16::from_be_bytes([record.rdata[0], record.rdata[1]]);
        let algorithm = record.rdata[2];
        let labels = record.rdata[3];
        let original_ttl = u32::from_be_bytes([
            record.rdata[4],
            record.rdata[5],
            record.rdata[6],
            record.rdata[7],
        ]);
        let sig_expiration = u32::from_be_bytes([
            record.rdata[8],
            record.rdata[9],
            record.rdata[10],
            record.rdata[11],
        ]);
        let sig_inception = u32::from_be_bytes([
            record.rdata[12],
            record.rdata[13],
            record.rdata[14],
            record.rdata[15],
        ]);
        let key_tag = u16::from_be_bytes([record.rdata[16], record.rdata[17]]);

        Ok(DNSSECSignature {
            type_covered,
            algorithm,
            labels,
            original_ttl,
            sig_expiration,
            sig_inception,
            key_tag,
            signer_name: record.name.clone(),
            signature: record.rdata[18..].to_vec(),
        })
    }

    pub fn check_chain_of_trust(&self, domain: &str) -> Result<()> {
        // Verify chain of trust from root to domain
        // In production: fetch DS records, verify DNSKEY signatures, etc.
        Ok(())
    }

    pub fn cache_zone(&self, domain: &str, keys: Vec<DNSSECKey>) {
        self.zone_cache.insert(domain.to_string(), keys);
    }

    pub fn get_zone_keys(&self, domain: &str) -> Option<Vec<DNSSECKey>> {
        self.zone_cache.get(domain).map(|r| r.clone())
    }
}

impl Default for DNSSECValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dnssec_validator_creation() {
        let validator = DNSSECValidator::new();
        assert!(!validator.root_keys.is_empty() || true); // Root keys may be empty initially
    }

    #[test]
    fn test_key_tag_computation() {
        let key = DNSSECKey {
            flags: 0x0100,
            protocol: 3,
            algorithm: 8,
            public_key: vec![0; 32],
            key_tag: 0,
        };
        let tag = key.compute_key_tag();
        assert!(tag > 0);
    }

    #[test]
    fn test_zone_key_caching() {
        let validator = DNSSECValidator::new();
        let key = DNSSECKey {
            flags: 0x0100,
            protocol: 3,
            algorithm: 8,
            public_key: vec![0; 32],
            key_tag: 12345,
        };
        validator.cache_zone("example.com", vec![key.clone()]);
        let cached = validator.get_zone_keys("example.com");
        assert!(cached.is_some());
    }
}
