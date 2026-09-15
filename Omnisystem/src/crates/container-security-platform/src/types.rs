//! Container image vulnerability scanning model.
//!
//! This models the scan-result/policy-gate protocol only — no real CVE
//! database or registry is contacted. Findings are supplied by the caller
//! (in tests, by fixtures) exactly as a real scanner client would decode
//! them from a scan report.

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// CVE severity, ordered low to critical so `Ord` comparisons work
/// ("at least HIGH" style gates).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    /// Negligible/informational.
    Low,
    /// Should be tracked but not urgent.
    Medium,
    /// Should be remediated soon.
    High,
    /// Actively exploitable / severe impact.
    Critical,
}

impl FromStr for Severity {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        match s.to_ascii_uppercase().as_str() {
            "LOW" => Ok(Severity::Low),
            "MEDIUM" => Ok(Severity::Medium),
            "HIGH" => Ok(Severity::High),
            "CRITICAL" => Ok(Severity::Critical),
            other => Err(Error::InvalidSeverity(other.to_string())),
        }
    }
}

/// A single vulnerability finding against one package inside a scanned
/// image layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// CVE identifier, e.g. "CVE-2024-12345".
    pub cve_id: String,
    /// Severity as reported by the scanner.
    pub severity: Severity,
    /// The package the vulnerability was found in, e.g. "openssl".
    pub affected_package: String,
    /// The installed version of the affected package.
    pub installed_version: String,
    /// The version that fixes the vulnerability, if a fix is available.
    pub fixed_version: Option<String>,
}

/// The full result of scanning one image.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    /// Image reference that was scanned, e.g. "myapp:1.2.3".
    pub image: String,
    /// Every finding the scan produced.
    pub findings: Vec<Finding>,
}

impl ScanResult {
    /// Findings at or above the given severity.
    pub fn findings_at_least(&self, min: Severity) -> Vec<&Finding> {
        self.findings.iter().filter(|f| f.severity >= min).collect()
    }

    /// Count findings by severity.
    pub fn severity_counts(&self) -> (usize, usize, usize, usize) {
        let mut counts = (0, 0, 0, 0); // low, medium, high, critical
        for f in &self.findings {
            match f.severity {
                Severity::Low => counts.0 += 1,
                Severity::Medium => counts.1 += 1,
                Severity::High => counts.2 += 1,
                Severity::Critical => counts.3 += 1,
            }
        }
        counts
    }

    /// Findings that have no fix available yet.
    pub fn unfixable(&self) -> Vec<&Finding> {
        self.findings.iter().filter(|f| f.fixed_version.is_none()).collect()
    }
}

/// The outcome of running a [`crate::gate::PolicyGate`] against a scan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GateOutcome {
    /// No finding violated the policy.
    Pass,
    /// At least one finding violated the policy; the offending CVE ids are
    /// listed for the caller to act on.
    Blocked { violating_cve_ids: Vec<String> },
}

impl GateOutcome {
    /// True if the gate passed.
    pub fn passed(&self) -> bool {
        matches!(self, GateOutcome::Pass)
    }
}
