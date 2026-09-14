/// Query Processing Pipeline Stages

use aether_dns_core::query::DNSQuery;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProcessingStage {
    Validation,
    PolicyCheck,
    CacheLookup,
    ThreatAnalysis,
    Resolution,
    ResponseBuilding,
    Logging,
}

impl ProcessingStage {
    pub fn to_string(&self) -> String {
        match self {
            ProcessingStage::Validation => "Validation".to_string(),
            ProcessingStage::PolicyCheck => "PolicyCheck".to_string(),
            ProcessingStage::CacheLookup => "CacheLookup".to_string(),
            ProcessingStage::ThreatAnalysis => "ThreatAnalysis".to_string(),
            ProcessingStage::Resolution => "Resolution".to_string(),
            ProcessingStage::ResponseBuilding => "ResponseBuilding".to_string(),
            ProcessingStage::Logging => "Logging".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProcessingContext {
    pub query: DNSQuery,
    pub current_stage: ProcessingStage,
    pub started_at: DateTime<Utc>,
    pub stage_times: Vec<(ProcessingStage, u128)>, // (stage, duration_ms)
    pub metadata: std::collections::HashMap<String, String>,
}

impl ProcessingContext {
    pub fn new(query: DNSQuery) -> Self {
        ProcessingContext {
            query,
            current_stage: ProcessingStage::Validation,
            started_at: Utc::now(),
            stage_times: Vec::new(),
            metadata: std::collections::HashMap::new(),
        }
    }

    pub fn move_to_stage(&mut self, stage: ProcessingStage) {
        self.current_stage = stage;
        tracing::debug!("Moving to stage: {}", stage.to_string());
    }

    pub fn record_stage_time(&mut self, stage: ProcessingStage, duration_ms: u128) {
        self.stage_times.push((stage, duration_ms));
    }

    pub fn total_elapsed_ms(&self) -> u128 {
        Utc::now()
            .signed_duration_since(self.started_at)
            .num_milliseconds() as u128
    }

    pub fn set_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn get_metadata(&self, key: &str) -> Option<String> {
        self.metadata.get(key).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aether_dns_core::protocol::RecordType;
    use aether_dns_core::query::QuerySource;

    #[test]
    fn test_processing_context_creation() {
        let query = DNSQuery::new(
            "example.com".to_string(),
            RecordType::A,
            QuerySource::UDP,
            "127.0.0.1".to_string(),
        );
        let context = ProcessingContext::new(query);
        assert_eq!(context.current_stage, ProcessingStage::Validation);
    }

    #[test]
    fn test_stage_transition() {
        let query = DNSQuery::new(
            "example.com".to_string(),
            RecordType::A,
            QuerySource::UDP,
            "127.0.0.1".to_string(),
        );
        let mut context = ProcessingContext::new(query);
        context.move_to_stage(ProcessingStage::CacheLookup);
        assert_eq!(context.current_stage, ProcessingStage::CacheLookup);
    }

    #[test]
    fn test_metadata_operations() {
        let query = DNSQuery::new(
            "example.com".to_string(),
            RecordType::A,
            QuerySource::UDP,
            "127.0.0.1".to_string(),
        );
        let mut context = ProcessingContext::new(query);
        context.set_metadata("user_id".to_string(), "user123".to_string());
        assert_eq!(context.get_metadata("user_id"), Some("user123".to_string()));
    }
}
