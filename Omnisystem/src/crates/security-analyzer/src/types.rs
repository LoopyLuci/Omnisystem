//! Static analysis domain model: source/config files, rules, and findings.
//!
//! Distinct from `container-security-platform`: that crate's unit of
//! analysis is a built container *image* and third-party package CVEs
//! inside it. This crate's unit of analysis is a source or config *file*
//! in a repository, checked against pattern-based rules for insecure
//! coding/config practices (hardcoded secrets, disabled TLS verification,
//! overly permissive permissions, etc.) — no image, no CVE database.

use serde::{Deserialize, Serialize};

/// Severity of a rule violation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    /// Style/best-practice only.
    Info,
    /// Should be fixed.
    Warning,
    /// Should be fixed promptly.
    Error,
    /// Exploitable; block the change.
    Critical,
}

/// A single static-analysis rule: a substring/pattern to flag when found
/// in a file's content, with a severity and message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    /// Stable rule id, e.g. "SEC001".
    pub id: String,
    /// Literal pattern to search for in file content. (A real
    /// implementation would use a regex/AST matcher; this models the
    /// rule-evaluation protocol with substring matching, which is enough
    /// to exercise real pass/fail decisions deterministically.)
    pub pattern: String,
    /// Severity if the pattern is found.
    pub severity: Severity,
    /// Human-readable explanation shown in findings.
    pub message: String,
}

impl Rule {
    /// Define a new rule.
    pub fn new(id: impl Into<String>, pattern: impl Into<String>, severity: Severity, message: impl Into<String>) -> Self {
        Self { id: id.into(), pattern: pattern.into(), severity, message: message.into() }
    }
}

/// A source/config file to analyze: a path plus its content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceFile {
    /// Repository-relative path.
    pub path: String,
    /// File content.
    pub content: String,
}

/// A single rule violation found in a file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// File the violation was found in.
    pub file_path: String,
    /// Rule that matched.
    pub rule_id: String,
    /// Severity, copied from the rule at match time.
    pub severity: Severity,
    /// Message, copied from the rule at match time.
    pub message: String,
    /// 1-based line number the pattern was found on.
    pub line: usize,
}

/// The full result of analyzing a set of files.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnalysisReport {
    /// Every finding across every analyzed file.
    pub findings: Vec<Finding>,
}

impl AnalysisReport {
    /// Findings at or above a severity.
    pub fn at_least(&self, min: Severity) -> Vec<&Finding> {
        self.findings.iter().filter(|f| f.severity >= min).collect()
    }

    /// True if any finding is Critical.
    pub fn has_critical(&self) -> bool {
        self.findings.iter().any(|f| f.severity == Severity::Critical)
    }
}
