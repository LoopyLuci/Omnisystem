//! Integration test: a small fixture repo scanned with default + custom
//! rules.

use security_analyzer::{Analyzer, Rule, Severity, SourceFile};

#[test]
fn repo_scan_flags_expected_violations_only() {
    let mut analyzer = Analyzer::with_default_rules();
    analyzer.register(Rule::new("CUSTOM001", "chmod 777", Severity::Error, "overly permissive file mode"));

    let files = vec![
        SourceFile {
            path: "src/db.py".into(),
            content: "password = \"placeholder-not-real\"\nconn = connect(password)\n".into(),
        },
        SourceFile {
            path: "src/http.py".into(),
            content: "resp = requests.get(url, verify=False)\n".into(),
        },
        SourceFile {
            path: "scripts/setup.sh".into(),
            content: "#!/bin/sh\nchmod 777 /var/data\n".into(),
        },
        SourceFile { path: "src/util.py".into(), content: "def helper():\n    return 42\n".into() },
    ];

    let report = analyzer.analyze(&files);

    // 3 files have exactly one violation each; util.py is clean.
    assert_eq!(report.findings.len(), 3);
    assert!(report.has_critical());

    let critical_files: Vec<_> = report.at_least(Severity::Critical).iter().map(|f| f.file_path.clone()).collect();
    assert_eq!(critical_files, vec!["src/db.py".to_string()]);

    let custom_hit = report.findings.iter().find(|f| f.rule_id == "CUSTOM001").unwrap();
    assert_eq!(custom_hit.file_path, "scripts/setup.sh");
    assert_eq!(custom_hit.line, 2);
}
