//! The rule-based analyzer: scan files against registered rules.

use crate::error::{Error, Result};
use crate::types::{AnalysisReport, Finding, Rule, SourceFile};
use std::collections::BTreeMap;

/// Holds registered rules and runs them against source files.
#[derive(Debug, Default)]
pub struct Analyzer {
    rules: BTreeMap<String, Rule>,
}

impl Analyzer {
    /// Create an analyzer with no rules registered.
    pub fn new() -> Self {
        Self::default()
    }

    /// An analyzer preloaded with a small built-in rule set covering
    /// common insecure patterns.
    pub fn with_default_rules() -> Self {
        let mut a = Self::new();
        a.register(Rule::new(
            "SEC001",
            "password = \"",
            crate::types::Severity::Critical,
            "hardcoded password literal",
        ));
        a.register(Rule::new(
            "SEC002",
            "verify=False",
            crate::types::Severity::Error,
            "TLS certificate verification disabled",
        ));
        a.register(Rule::new(
            "SEC003",
            "eval(",
            crate::types::Severity::Warning,
            "use of eval() on potentially untrusted input",
        ));
        a
    }

    /// Register a rule.
    pub fn register(&mut self, rule: Rule) {
        self.rules.insert(rule.id.clone(), rule);
    }

    /// Analyze a single file, returning any findings.
    pub fn analyze_file(&self, file: &SourceFile) -> Vec<Finding> {
        let mut findings = Vec::new();
        for (line_no, line) in file.content.lines().enumerate() {
            for rule in self.rules.values() {
                if line.contains(&rule.pattern) {
                    findings.push(Finding {
                        file_path: file.path.clone(),
                        rule_id: rule.id.clone(),
                        severity: rule.severity,
                        message: rule.message.clone(),
                        line: line_no + 1,
                    });
                }
            }
        }
        findings
    }

    /// Analyze a batch of files, aggregating findings into one report.
    pub fn analyze(&self, files: &[SourceFile]) -> AnalysisReport {
        let mut findings = Vec::new();
        for file in files {
            findings.extend(self.analyze_file(file));
        }
        AnalysisReport { findings }
    }

    /// Look up whether a rule id is registered (useful for verifying
    /// config before a scan).
    pub fn has_rule(&self, id: &str) -> Result<()> {
        if self.rules.contains_key(id) {
            Ok(())
        } else {
            Err(Error::UnknownRule(id.to_string()))
        }
    }
}
