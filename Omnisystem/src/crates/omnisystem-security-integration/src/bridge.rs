//! The integration bridge: ingest findings from any security-domain
//! source, track their triage state, and compute aggregate posture.

use crate::error::{Error, Result};
use crate::types::{FindingState, PostureSummary, SecurityFinding, Severity, TrackedFinding};
use std::collections::BTreeMap;

/// Aggregates [`SecurityFinding`]s from multiple sources into one tracked
/// view with a unified triage lifecycle and posture rollup. Deliberately
/// has no detection logic of its own: it only reconciles what upstream
/// crates report, the same bridging role `omnidocker-state-manager` plays
/// for Docker-side state.
#[derive(Debug, Default)]
pub struct SecurityIntegrationBridge {
    findings: BTreeMap<String, TrackedFinding>,
}

impl SecurityIntegrationBridge {
    /// Create an empty bridge.
    pub fn new() -> Self {
        Self::default()
    }

    /// Ingest a finding. If a finding with the same id was already
    /// ingested, its severity/summary/resource are refreshed but its
    /// current triage state (e.g. `Acknowledged`) is preserved — a
    /// re-scan re-reporting the same issue should not reset triage work
    /// already done on it.
    pub fn ingest(&mut self, finding: SecurityFinding) {
        match self.findings.get_mut(&finding.id) {
            Some(existing) => existing.finding = finding,
            None => {
                let id = finding.id.clone();
                self.findings.insert(id, TrackedFinding { finding, state: FindingState::Open });
            }
        }
    }

    /// Mark a finding acknowledged.
    pub fn acknowledge(&mut self, id: &str, by: impl Into<String>) -> Result<()> {
        self.transition(id, FindingState::Acknowledged { by: by.into() })
    }

    /// Mark a finding waived (accepted risk).
    pub fn waive(&mut self, id: &str, reason: impl Into<String>) -> Result<()> {
        self.transition(id, FindingState::Waived { reason: reason.into() })
    }

    /// Mark a finding resolved.
    pub fn resolve(&mut self, id: &str) -> Result<()> {
        self.transition(id, FindingState::Resolved)
    }

    fn transition(&mut self, id: &str, state: FindingState) -> Result<()> {
        let tracked = self.findings.get_mut(id).ok_or_else(|| Error::UnknownFinding(id.to_string()))?;
        tracked.state = state;
        Ok(())
    }

    /// Look up a tracked finding by id.
    pub fn get(&self, id: &str) -> Result<&TrackedFinding> {
        self.findings.get(id).ok_or_else(|| Error::UnknownFinding(id.to_string()))
    }

    /// All findings still counted as open (Open or Acknowledged).
    pub fn open_findings(&self) -> Vec<&TrackedFinding> {
        self.findings.values().filter(|t| t.state.counts_as_open()).collect()
    }

    /// Compute aggregate posture across all open findings.
    pub fn posture(&self) -> PostureSummary {
        let mut counts = (0, 0, 0, 0, 0);
        let mut risk_score = 0u32;
        for t in self.open_findings() {
            match t.finding.severity {
                Severity::Info => counts.0 += 1,
                Severity::Low => counts.1 += 1,
                Severity::Medium => counts.2 += 1,
                Severity::High => counts.3 += 1,
                Severity::Critical => counts.4 += 1,
            }
            risk_score += t.finding.severity.weight();
        }
        PostureSummary { counts_by_severity: counts, risk_score }
    }
}
