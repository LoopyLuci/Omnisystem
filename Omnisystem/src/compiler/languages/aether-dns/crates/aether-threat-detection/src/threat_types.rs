/// Threat Types and Classifications

use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThreatLevel {
    None,       // 0.0-0.2
    Low,        // 0.2-0.4
    Medium,     // 0.4-0.6
    High,       // 0.6-0.8
    Critical,   // 0.8-1.0
}

impl ThreatLevel {
    pub fn from_score(score: f64) -> Self {
        match score {
            s if s < 0.2 => ThreatLevel::None,
            s if s < 0.4 => ThreatLevel::Low,
            s if s < 0.6 => ThreatLevel::Medium,
            s if s < 0.8 => ThreatLevel::High,
            _ => ThreatLevel::Critical,
        }
    }

    pub fn to_score(&self) -> f64 {
        match self {
            ThreatLevel::None => 0.0,
            ThreatLevel::Low => 0.3,
            ThreatLevel::Medium => 0.5,
            ThreatLevel::High => 0.7,
            ThreatLevel::Critical => 0.9,
        }
    }

    pub fn block_threshold() -> f64 {
        0.75 // High/Critical
    }

    pub fn alert_threshold() -> f64 {
        0.55 // Medium+
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThreatType {
    DnsAmplification,    // Large response to small query
    FastFlux,            // Rapidly changing DNS responses
    DomainGeneration,    // Algorithmically generated domains
    SlowLoris,           // Many small requests over time
    CommandControl,      // Known C2 domain patterns
    BotnetCommand,       // Botnet C&C domain patterns
    Phishing,            // Lookalike domains
    Malware,             // Known malware C2
    Botnet,              // Botnet activity signatures
    DataExfiltration,    // Suspicious outbound patterns
    RateLimitAbuse,      // Excessive query rate
    TunnellingAttempt,   // DNS tunneling/exfiltration
    CachePoisoning,      // DNSSEC validation bypass attempts
    C2Command,           // C&C command patterns
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatIndicator {
    pub threat_type: ThreatType,
    pub confidence: f64,  // 0.0-1.0
    pub evidence: String,
    pub detected_at: String,
}

#[derive(Debug, Clone)]
pub struct ThreatAnalysis {
    pub threat_level: ThreatLevel,
    pub base_score: f64,
    pub indicators: Vec<ThreatIndicator>,
    pub should_block: bool,
    pub should_log: bool,
    pub analysis_time_ms: u64,
}

impl ThreatAnalysis {
    pub fn new() -> Self {
        ThreatAnalysis {
            threat_level: ThreatLevel::None,
            base_score: 0.0,
            indicators: Vec::new(),
            should_block: false,
            should_log: false,
            analysis_time_ms: 0,
        }
    }

    pub fn add_indicator(&mut self, indicator: ThreatIndicator) {
        self.indicators.push(indicator);
        self.recalculate_score();
    }

    fn recalculate_score(&mut self) {
        if self.indicators.is_empty() {
            self.base_score = 0.0;
        } else {
            // Weighted average of indicator confidences
            let total_confidence: f64 = self.indicators.iter().map(|i| i.confidence).sum();
            self.base_score = total_confidence / self.indicators.len() as f64;
        }

        self.threat_level = ThreatLevel::from_score(self.base_score);
        self.should_block = self.base_score > ThreatLevel::block_threshold();
        self.should_log = self.base_score > ThreatLevel::alert_threshold();
    }
}

impl Default for ThreatAnalysis {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_threat_level_from_score() {
        assert_eq!(ThreatLevel::from_score(0.1), ThreatLevel::None);
        assert_eq!(ThreatLevel::from_score(0.3), ThreatLevel::Low);
        assert_eq!(ThreatLevel::from_score(0.5), ThreatLevel::Medium);
        assert_eq!(ThreatLevel::from_score(0.7), ThreatLevel::High);
        assert_eq!(ThreatLevel::from_score(0.9), ThreatLevel::Critical);
    }

    #[test]
    fn test_threat_analysis_indicators() {
        let mut analysis = ThreatAnalysis::new();
        assert_eq!(analysis.base_score, 0.0);

        analysis.add_indicator(ThreatIndicator {
            threat_type: ThreatType::DomainGeneration,
            confidence: 0.8,
            evidence: "DGA pattern matched".to_string(),
            detected_at: "2026-06-11T00:00:00Z".to_string(),
        });

        assert!(analysis.base_score > 0.0);
        assert!(analysis.should_block);
    }

    #[test]
    fn test_thresholds() {
        assert!(ThreatLevel::block_threshold() > ThreatLevel::alert_threshold());
        assert_eq!(ThreatLevel::block_threshold(), 0.75);
        assert_eq!(ThreatLevel::alert_threshold(), 0.55);
    }
}
