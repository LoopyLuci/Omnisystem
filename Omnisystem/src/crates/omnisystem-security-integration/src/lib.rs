//! omnisystem-security-integration: cross-source security posture bridge.
//!
//! This crate does not detect vulnerabilities, evaluate RBAC policy, scan
//! containers, or check compliance controls itself — those are the jobs of
//! `rbac-authorization-engine`, `container-security-platform`,
//! `secret-management-integration`, `compliance-framework`, and
//! `security-analyzer`. Instead, mirroring the role
//! `omnidocker-state-manager` plays for Docker state, this crate is the
//! Omnisystem-side bridge: it ingests normalized [`types::SecurityFinding`]s
//! from any of those sources, tracks a shared triage lifecycle
//! (Open -> Acknowledged/Waived/Resolved) across all of them, and computes
//! one aggregate [`types::PostureSummary`] risk score regardless of which
//! source a finding came from.

pub mod bridge;
pub mod error;
pub mod types;

pub use bridge::SecurityIntegrationBridge;
pub use error::{Error, Result};
pub use types::*;

#[cfg(test)]
mod tests {
    use super::*;

    fn finding(id: &str, source: SecuritySource, severity: Severity) -> SecurityFinding {
        SecurityFinding { id: id.to_string(), source, severity, summary: "test".into(), resource: "r1".into() }
    }

    #[test]
    fn ingest_starts_open() {
        let mut bridge = SecurityIntegrationBridge::new();
        bridge.ingest(finding("F1", SecuritySource::ContainerScan, Severity::High));
        assert_eq!(bridge.get("F1").unwrap().state, FindingState::Open);
    }

    #[test]
    fn unknown_finding_lookup_errors() {
        let bridge = SecurityIntegrationBridge::new();
        assert!(matches!(bridge.get("ghost"), Err(Error::UnknownFinding(_))));
    }

    #[test]
    fn acknowledge_transitions_state() {
        let mut bridge = SecurityIntegrationBridge::new();
        bridge.ingest(finding("F1", SecuritySource::Rbac, Severity::Medium));
        bridge.acknowledge("F1", "alice").unwrap();
        assert_eq!(bridge.get("F1").unwrap().state, FindingState::Acknowledged { by: "alice".into() });
    }

    #[test]
    fn waived_findings_excluded_from_open() {
        let mut bridge = SecurityIntegrationBridge::new();
        bridge.ingest(finding("F1", SecuritySource::Compliance, Severity::High));
        bridge.waive("F1", "accepted risk, compensating control in place").unwrap();
        assert!(bridge.open_findings().is_empty());
    }

    #[test]
    fn resolved_findings_excluded_from_posture() {
        let mut bridge = SecurityIntegrationBridge::new();
        bridge.ingest(finding("F1", SecuritySource::SecretManagement, Severity::Critical));
        bridge.resolve("F1").unwrap();
        assert_eq!(bridge.posture().risk_score, 0);
    }

    #[test]
    fn acknowledged_still_counts_toward_posture() {
        let mut bridge = SecurityIntegrationBridge::new();
        bridge.ingest(finding("F1", SecuritySource::StaticAnalysis, Severity::High));
        bridge.acknowledge("F1", "bob").unwrap();
        assert_eq!(bridge.posture().risk_score, Severity::High.weight());
    }

    #[test]
    fn posture_aggregates_across_sources() {
        let mut bridge = SecurityIntegrationBridge::new();
        bridge.ingest(finding("F1", SecuritySource::ContainerScan, Severity::Critical));
        bridge.ingest(finding("F2", SecuritySource::Rbac, Severity::Low));
        bridge.ingest(finding("F3", SecuritySource::Compliance, Severity::Medium));
        let posture = bridge.posture();
        assert_eq!(posture.counts_by_severity, (0, 1, 1, 0, 1));
        assert_eq!(posture.risk_score, Severity::Critical.weight() + Severity::Low.weight() + Severity::Medium.weight());
    }

    #[test]
    fn re_ingest_refreshes_finding_but_preserves_triage_state() {
        let mut bridge = SecurityIntegrationBridge::new();
        bridge.ingest(finding("F1", SecuritySource::ContainerScan, Severity::Medium));
        bridge.acknowledge("F1", "carol").unwrap();
        // A re-scan reports the same finding id with escalated severity.
        bridge.ingest(finding("F1", SecuritySource::ContainerScan, Severity::Critical));
        let tracked = bridge.get("F1").unwrap();
        assert_eq!(tracked.finding.severity, Severity::Critical);
        assert_eq!(tracked.state, FindingState::Acknowledged { by: "carol".into() });
    }

    #[test]
    fn transition_on_unknown_finding_errors() {
        let mut bridge = SecurityIntegrationBridge::new();
        assert!(matches!(bridge.acknowledge("ghost", "x"), Err(Error::UnknownFinding(_))));
    }
}
