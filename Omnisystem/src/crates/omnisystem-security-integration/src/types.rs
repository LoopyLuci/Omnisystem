//! Cross-source security finding model.

use serde::{Deserialize, Serialize};

/// Which security-domain crate/system a finding originated from. This
/// crate does not itself detect anything in these domains (that's the job
/// of `rbac-authorization-engine`, `container-security-platform`,
/// `secret-management-integration`, `compliance-framework`, and
/// `security-analyzer` respectively) — it only aggregates what they report.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SecuritySource {
    /// From RBAC policy evaluation (e.g. an overly-broad wildcard grant).
    Rbac,
    /// From container image vulnerability scanning.
    ContainerScan,
    /// From secret rotation/lifecycle tracking.
    SecretManagement,
    /// From compliance control evaluation.
    Compliance,
    /// From static code/config analysis.
    StaticAnalysis,
}

/// Severity, ordered low to critical.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    /// Informational only.
    Info,
    /// Should be tracked.
    Low,
    /// Should be remediated.
    Medium,
    /// Should be remediated soon.
    High,
    /// Requires immediate attention.
    Critical,
}

impl Severity {
    /// Numeric weight used in posture risk scoring.
    pub fn weight(self) -> u32 {
        match self {
            Severity::Info => 0,
            Severity::Low => 1,
            Severity::Medium => 3,
            Severity::High => 7,
            Severity::Critical => 15,
        }
    }
}

/// The disposition of an ingested finding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FindingState {
    /// Newly ingested, not yet triaged.
    Open,
    /// Triaged and accepted as a real, tracked issue, by whom.
    Acknowledged { by: String },
    /// Explicitly accepted as a risk that will not be fixed, with a reason.
    Waived { reason: String },
    /// Confirmed fixed.
    Resolved,
}

impl FindingState {
    /// Whether this state still counts toward open security posture (i.e.
    /// not waived or resolved).
    pub fn counts_as_open(&self) -> bool {
        matches!(self, FindingState::Open | FindingState::Acknowledged { .. })
    }
}

/// A single normalized finding bridged in from one of the security-domain
/// crates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityFinding {
    /// Stable id, assigned by the originating source (e.g. a CVE id, a
    /// control id, a policy-violation id).
    pub id: String,
    /// Which system reported it.
    pub source: SecuritySource,
    /// Severity as reported.
    pub severity: Severity,
    /// Human-readable summary.
    pub summary: String,
    /// The resource the finding is about (image ref, role name, secret
    /// name, control id, file path, etc.).
    pub resource: String,
}

/// A tracked finding: the original report plus its current disposition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackedFinding {
    /// The normalized finding.
    pub finding: SecurityFinding,
    /// Current triage state.
    pub state: FindingState,
}

/// Aggregate security posture across all ingested, still-open findings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PostureSummary {
    /// Count of open (Open or Acknowledged) findings by severity, indexed
    /// as (info, low, medium, high, critical).
    pub counts_by_severity: (usize, usize, usize, usize, usize),
    /// Sum of severity weights across all open findings.
    pub risk_score: u32,
}
