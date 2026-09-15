//! Demo CLI: scan a couple of fixture files with the default rule set.

use security_analyzer::{Analyzer, SourceFile};

fn main() {
    let analyzer = Analyzer::with_default_rules();
    let files = vec![
        SourceFile { path: "config.py".into(), content: "password = \"placeholder\"".into() },
        SourceFile { path: "client.py".into(), content: "requests.get(url, verify=False)".into() },
    ];
    let report = analyzer.analyze(&files);
    for f in &report.findings {
        println!("{}:{} [{:?}] {} ({})", f.file_path, f.line, f.severity, f.message, f.rule_id);
    }
}
