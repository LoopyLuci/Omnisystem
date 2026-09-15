//! security-analyzer: static analysis of source/config files.
//!
//! Distinct from `container-security-platform` (which scans built
//! container *images* for third-party CVEs): this crate scans source and
//! config *file content* against a registered set of pattern-based
//! [`types::Rule`]s (hardcoded secrets, disabled TLS verification, unsafe
//! `eval` use, etc.) and produces per-file, per-line [`types::Finding`]s.

pub mod analyzer;
pub mod error;
pub mod types;

pub use analyzer::Analyzer;
pub use error::{Error, Result};
pub use types::*;

#[cfg(test)]
mod tests {
    use super::*;

    fn file(path: &str, content: &str) -> SourceFile {
        SourceFile { path: path.to_string(), content: content.to_string() }
    }

    #[test]
    fn no_rules_no_findings() {
        let analyzer = Analyzer::new();
        let findings = analyzer.analyze_file(&file("a.py", "password = \"hunter2\""));
        assert!(findings.is_empty());
    }

    #[test]
    fn default_rules_catch_hardcoded_password() {
        let analyzer = Analyzer::with_default_rules();
        let findings = analyzer.analyze_file(&file("config.py", "password = \"hunter2\""));
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "SEC001");
        assert_eq!(findings[0].severity, Severity::Critical);
    }

    #[test]
    fn line_number_is_reported_correctly() {
        let analyzer = Analyzer::with_default_rules();
        let content = "# comment\n# another\npassword = \"x\"";
        let findings = analyzer.analyze_file(&file("a.py", content));
        assert_eq!(findings[0].line, 3);
    }

    #[test]
    fn clean_file_has_no_findings() {
        let analyzer = Analyzer::with_default_rules();
        let findings = analyzer.analyze_file(&file("clean.py", "def add(a, b):\n    return a + b\n"));
        assert!(findings.is_empty());
    }

    #[test]
    fn multiple_rules_can_match_same_line() {
        let mut analyzer = Analyzer::new();
        analyzer.register(Rule::new("R1", "TODO", Severity::Info, "todo marker"));
        analyzer.register(Rule::new("R2", "eval(", Severity::Warning, "eval use"));
        let findings = analyzer.analyze_file(&file("a.py", "# TODO remove eval(x)"));
        assert_eq!(findings.len(), 2);
    }

    #[test]
    fn analyze_aggregates_across_files() {
        let analyzer = Analyzer::with_default_rules();
        let files = vec![
            file("a.py", "password = \"x\""),
            file("b.py", "requests.get(url, verify=False)"),
            file("c.py", "clean"),
        ];
        let report = analyzer.analyze(&files);
        assert_eq!(report.findings.len(), 2);
        assert!(report.has_critical());
    }

    #[test]
    fn at_least_filters_by_severity() {
        let analyzer = Analyzer::with_default_rules();
        let files = vec![file("a.py", "eval(x)"), file("b.py", "password = \"x\"")];
        let report = analyzer.analyze(&files);
        let severe = report.at_least(Severity::Error);
        assert_eq!(severe.len(), 1);
        assert_eq!(severe[0].rule_id, "SEC001");
    }

    #[test]
    fn has_rule_errors_on_unknown_id() {
        let analyzer = Analyzer::with_default_rules();
        assert!(analyzer.has_rule("SEC001").is_ok());
        assert!(matches!(analyzer.has_rule("GHOST"), Err(Error::UnknownRule(_))));
    }
}
