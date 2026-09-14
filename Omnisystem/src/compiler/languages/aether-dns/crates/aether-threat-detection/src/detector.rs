/// Threat Detector
/// Main threat detection orchestrator

use crate::threat_types::{ThreatAnalysis, ThreatIndicator, ThreatType};
use crate::classifier::ThreatClassifier;
use crate::fingerprint::Fingerprinter;
use crate::rate_limiter::RateLimiter;
use std::time::Instant;
use tracing::{debug, warn};

pub struct ThreatDetector {
    classifier: ThreatClassifier,
    fingerprinter: Fingerprinter,
    rate_limiter: RateLimiter,
}

impl ThreatDetector {
    pub fn new() -> Self {
        ThreatDetector {
            classifier: ThreatClassifier::new(),
            fingerprinter: Fingerprinter::new(),
            rate_limiter: RateLimiter::new(),
        }
    }

    pub async fn analyze(&self, domain: &str, source_ip: &str) -> ThreatAnalysis {
        let start = Instant::now();
        let mut analysis = ThreatAnalysis::new();

        debug!("Analyzing domain: {} from {}", domain, source_ip);

        // Check domain patterns
        if let Some(indicator) = self.classifier.classify_domain(domain) {
            analysis.add_indicator(indicator);
        }

        // Check known threats
        if let Some(indicator) = self.classifier.check_known_threats(domain) {
            analysis.add_indicator(indicator);
        }

        // Check fingerprint/signature
        if let Some(indicator) = self.fingerprinter.detect_signature(domain) {
            analysis.add_indicator(indicator);
        }

        // Check rate limiting
        if let Some(indicator) = self.rate_limiter.check_rate(source_ip) {
            analysis.add_indicator(indicator);
        }

        analysis.analysis_time_ms = start.elapsed().as_millis() as u64;

        if analysis.should_block {
            warn!(
                "Blocked query: {} from {} (score: {:.2})",
                domain, source_ip, analysis.base_score
            );
        }

        analysis
    }

    pub async fn analyze_batch(
        &self,
        queries: Vec<(&str, &str)>,
    ) -> Vec<ThreatAnalysis> {
        let mut results = Vec::new();
        for (domain, source_ip) in queries {
            let analysis = self.analyze(domain, source_ip).await;
            results.push(analysis);
        }
        results
    }

    pub fn get_threat_stats(&self) -> ThreatDetectorStats {
        let stats = self.classifier.get_stats();
        let total_analyzed = stats.iter().map(|(_, count)| count).sum::<u64>();
        let blocked_count = stats.iter().filter(|(name, _)| name.contains("blocked")).map(|(_, count)| count).sum::<u64>();
        let flagged_count = stats.len() as u64;

        ThreatDetectorStats {
            total_analyzed,
            blocked_count,
            flagged_count,
            rate_limited: self.rate_limiter.get_limited_ips(),
        }
    }
}

impl Default for ThreatDetector {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ThreatDetectorStats {
    pub total_analyzed: u64,
    pub blocked_count: u64,
    pub flagged_count: u64,
    pub rate_limited: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_detector_creation() {
        let detector = ThreatDetector::new();
        let stats = detector.get_threat_stats();
        assert_eq!(stats.total_analyzed, 0);
    }

    #[tokio::test]
    async fn test_analyze_safe_domain() {
        let detector = ThreatDetector::new();
        let analysis = detector.analyze("google.com", "192.168.1.1").await;
        assert!(!analysis.should_block);
    }

    #[tokio::test]
    async fn test_analyze_batch() {
        let detector = ThreatDetector::new();
        let queries = vec![
            ("google.com", "192.168.1.1"),
            ("example.com", "192.168.1.2"),
        ];
        let results = detector.analyze_batch(queries).await;
        assert_eq!(results.len(), 2);
    }
}
