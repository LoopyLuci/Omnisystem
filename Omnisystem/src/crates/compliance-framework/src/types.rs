//! Compliance control model: control definitions, evidence, and findings.

use serde::{Deserialize, Serialize};

/// A single compliance control (e.g. one requirement from SOC2, ISO 27001,
/// or an internal policy).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Control {
    /// Stable identifier, e.g. "AC-2" or "SOC2-CC6.1".
    pub id: String,
    /// Human-readable description of what the control requires.
    pub description: String,
    /// Evidence types that must be present for this control to pass, e.g.
    /// "access_review_log", "encryption_at_rest_config".
    pub required_evidence: Vec<String>,
}

impl Control {
    /// Define a new control.
    pub fn new(id: impl Into<String>, description: impl Into<String>) -> Self {
        Self { id: id.into(), description: description.into(), required_evidence: Vec::new() }
    }

    /// Declare a required evidence type (builder-style).
    pub fn requires(mut self, evidence_type: impl Into<String>) -> Self {
        self.required_evidence.push(evidence_type.into());
        self
    }
}

/// A single piece of evidence submitted against a control.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    /// Which control this evidence is submitted for.
    pub control_id: String,
    /// The evidence type, matched against `Control::required_evidence`.
    pub evidence_type: String,
    /// Whether the evidence, once reviewed, actually satisfies the
    /// requirement (e.g. an access review log that exists but shows stale
    /// reviews would be `valid: false`).
    pub valid: bool,
}

/// The evaluated status of a single control after evidence is applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControlStatus {
    /// All required evidence types are present and valid.
    Pass,
    /// Some, but not all, required evidence types are present and valid.
    Partial,
    /// No required evidence is present and valid (or the control has no
    /// evidence submitted at all).
    Fail,
}

/// The result of evaluating one control.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlResult {
    /// The control this result is for.
    pub control_id: String,
    /// Pass/partial/fail status.
    pub status: ControlStatus,
    /// Required evidence types that were satisfied by valid evidence.
    pub satisfied_evidence: Vec<String>,
    /// Required evidence types still missing or invalid.
    pub missing_evidence: Vec<String>,
}

/// A full compliance report across a set of controls.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
    /// Per-control results.
    pub results: Vec<ControlResult>,
}

impl ComplianceReport {
    /// Number of controls with each status.
    pub fn summary(&self) -> (usize, usize, usize) {
        let pass = self.results.iter().filter(|r| r.status == ControlStatus::Pass).count();
        let partial = self.results.iter().filter(|r| r.status == ControlStatus::Partial).count();
        let fail = self.results.iter().filter(|r| r.status == ControlStatus::Fail).count();
        (pass, partial, fail)
    }

    /// True only if every control fully passed.
    pub fn fully_compliant(&self) -> bool {
        self.results.iter().all(|r| r.status == ControlStatus::Pass)
    }
}
