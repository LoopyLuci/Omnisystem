//! Integration test: a mixed-source event feed shaped into summary +
//! paginated + filtered views consistently.

use security_console_ui::{filter_by_source, paginate, summarize, SourceEvent};

#[test]
fn dashboard_views_stay_consistent_across_shapes() {
    let events: Vec<SourceEvent> = vec![
        ("c1", "container-security-platform", 4),
        ("c2", "container-security-platform", 1),
        ("r1", "rbac-authorization-engine", 2),
        ("s1", "security-analyzer", 3),
        ("s2", "security-analyzer", 0),
    ]
    .into_iter()
    .map(|(id, source, severity)| SourceEvent { id: id.into(), source: source.into(), severity, title: id.into() })
    .collect();

    let summary = summarize(&events);
    assert_eq!(summary.total_events, 5);
    assert_eq!(summary.sources, vec![
        "container-security-platform".to_string(),
        "rbac-authorization-engine".to_string(),
        "security-analyzer".to_string(),
    ]);

    // Full pagination across two pages of size 2 covers every event exactly once.
    let page0 = paginate(&events, 0, 2);
    let page1 = paginate(&events, 1, 2);
    let page2 = paginate(&events, 2, 2);
    let mut all_ids: Vec<String> =
        page0.items.iter().chain(&page1.items).chain(&page2.items).map(|e| e.id.clone()).collect();
    all_ids.sort();
    let mut expected: Vec<String> = events.iter().map(|e| e.id.clone()).collect();
    expected.sort();
    assert_eq!(all_ids, expected);
    // First page is the two highest-severity events.
    assert_eq!(page0.items[0].id, "c1"); // severity 4

    let container_only = filter_by_source(&events, "container-security-platform");
    assert_eq!(container_only.len(), 2);
}
