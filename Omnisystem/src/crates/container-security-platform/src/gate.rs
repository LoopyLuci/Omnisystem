//! Policy gate: decide whether a scan result is acceptable to ship.

use crate::types::{GateOutcome, ScanResult, Severity};

/// A deployment gate: fail the build/deploy if any finding meets or
/// exceeds `block_at_or_above`, unless it's in the caller's accepted-risk
/// allowlist (`allowed_cve_ids`, e.g. for vulnerabilities under an
/// approved exception).
#[derive(Debug, Clone)]
pub struct PolicyGate {
    /// Minimum severity that blocks a scan from passing.
    pub block_at_or_above: Severity,
    /// CVE ids explicitly allowed through despite meeting the threshold.
    pub allowed_cve_ids: Vec<String>,
}

impl PolicyGate {
    /// A gate that blocks on any CRITICAL finding, with no exceptions.
    pub fn block_critical() -> Self {
        Self { block_at_or_above: Severity::Critical, allowed_cve_ids: Vec::new() }
    }

    /// Build a gate with a custom threshold.
    pub fn with_threshold(block_at_or_above: Severity) -> Self {
        Self { block_at_or_above, allowed_cve_ids: Vec::new() }
    }

    /// Allow a specific CVE through the gate regardless of severity
    /// (builder-style).
    pub fn allow(mut self, cve_id: impl Into<String>) -> Self {
        self.allowed_cve_ids.push(cve_id.into());
        self
    }

    /// Evaluate a scan result against this gate.
    pub fn evaluate(&self, scan: &ScanResult) -> GateOutcome {
        let violating: Vec<String> = scan
            .findings
            .iter()
            .filter(|f| f.severity >= self.block_at_or_above && !self.allowed_cve_ids.contains(&f.cve_id))
            .map(|f| f.cve_id.clone())
            .collect();
        if violating.is_empty() {
            GateOutcome::Pass
        } else {
            GateOutcome::Blocked { violating_cve_ids: violating }
        }
    }
}
