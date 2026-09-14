/// Fingerprint Detection
/// Signature-based threat detection

use crate::threat_types::{ThreatIndicator, ThreatType};
use blake3;

pub struct Fingerprinter {
    threat_signatures: Vec<ThreatSignature>,
}

pub struct ThreatSignature {
    pub name: String,
    pub hash: String,
    pub threat_type: ThreatType,
    pub confidence: f64,
}

impl Fingerprinter {
    pub fn new() -> Self {
        let threat_signatures = vec![
            ThreatSignature {
                name: "botnet_signature_1".to_string(),
                hash: "signature_pattern_1".to_string(),
                threat_type: ThreatType::Botnet,
                confidence: 0.85,
            },
            ThreatSignature {
                name: "malware_signature_1".to_string(),
                hash: "signature_pattern_2".to_string(),
                threat_type: ThreatType::Malware,
                confidence: 0.9,
            },
        ];

        Fingerprinter { threat_signatures }
    }

    pub fn detect_signature(&self, domain: &str) -> Option<ThreatIndicator> {
        let domain_hash = self.hash_domain(domain);

        for signature in &self.threat_signatures {
            // Simple pattern matching (in production, use YARA rules or similar)
            if self.pattern_match(&domain, &signature.name) {
                return Some(ThreatIndicator {
                    threat_type: signature.threat_type.clone(),
                    confidence: signature.confidence,
                    evidence: format!(
                        "Signature match: {} (hash: {})",
                        signature.name, domain_hash
                    ),
                    detected_at: chrono::Utc::now().to_rfc3339(),
                });
            }
        }

        None
    }

    pub fn add_signature(&mut self, signature: ThreatSignature) {
        self.threat_signatures.push(signature);
    }

    fn hash_domain(&self, domain: &str) -> String {
        let hash = blake3::hash(domain.as_bytes());
        hash.to_hex().to_string()[..16].to_string()
    }

    fn pattern_match(&self, domain: &str, pattern: &str) -> bool {
        // Placeholder pattern matching
        domain.contains(pattern) || domain.ends_with(&pattern.to_lowercase())
    }

    pub fn signature_count(&self) -> usize {
        self.threat_signatures.len()
    }
}

impl Default for Fingerprinter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fingerprinter_creation() {
        let fingerprinter = Fingerprinter::new();
        assert_eq!(fingerprinter.signature_count(), 2);
    }

    #[test]
    fn test_add_signature() {
        let mut fingerprinter = Fingerprinter::new();
        let sig = ThreatSignature {
            name: "new_sig".to_string(),
            hash: "hash123".to_string(),
            threat_type: ThreatType::Malware,
            confidence: 0.75,
        };
        fingerprinter.add_signature(sig);
        assert_eq!(fingerprinter.signature_count(), 3);
    }

    #[test]
    fn test_domain_hashing() {
        let fingerprinter = Fingerprinter::new();
        let hash1 = fingerprinter.hash_domain("example.com");
        let hash2 = fingerprinter.hash_domain("example.com");
        assert_eq!(hash1, hash2); // Deterministic
    }
}
