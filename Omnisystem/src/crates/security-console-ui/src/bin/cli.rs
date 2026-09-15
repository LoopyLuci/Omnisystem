//! Demo CLI: shape a fixture event list into a summary and first page.

use security_console_ui::{paginate, summarize, SourceEvent};

fn main() {
    let events = vec![
        SourceEvent { id: "1".into(), source: "container-security-platform".into(), severity: 4, title: "critical CVE".into() },
        SourceEvent { id: "2".into(), source: "security-analyzer".into(), severity: 2, title: "eval() use".into() },
    ];
    println!("{:?}", summarize(&events));
    println!("{:?}", paginate(&events, 0, 10));
}
