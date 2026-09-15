//! Data-shaping functions: turn a flat list of source events into the
//! summary and paginated views a dashboard actually renders.

use crate::types::{DashboardSummary, EventPage, SeverityBucket, SourceEvent};
use std::collections::BTreeSet;

/// Build the summary widget data for a set of events.
pub fn summarize(events: &[SourceEvent]) -> DashboardSummary {
    let mut counts = [0usize; 5];
    let mut sources: BTreeSet<String> = BTreeSet::new();
    for e in events {
        let idx = e.severity.min(4) as usize;
        counts[idx] += 1;
        sources.insert(e.source.clone());
    }
    let severity_buckets =
        counts.iter().enumerate().map(|(sev, &count)| SeverityBucket { severity: sev as u8, count }).collect();
    DashboardSummary { severity_buckets, sources: sources.into_iter().collect(), total_events: events.len() }
}

/// Sort events highest-severity-first (stable on id for a deterministic
/// display order among ties) and return the requested page.
pub fn paginate(events: &[SourceEvent], page: usize, page_size: usize) -> EventPage {
    let mut sorted: Vec<SourceEvent> = events.to_vec();
    sorted.sort_by(|a, b| b.severity.cmp(&a.severity).then_with(|| a.id.cmp(&b.id)));
    let total = sorted.len();
    let start = (page * page_size).min(total);
    let end = (start + page_size).min(total);
    EventPage { items: sorted[start..end].to_vec(), page, page_size, total }
}

/// Filter events down to a single source system, e.g. for a per-source
/// drill-down view.
pub fn filter_by_source<'a>(events: &'a [SourceEvent], source: &str) -> Vec<&'a SourceEvent> {
    events.iter().filter(|e| e.source == source).collect()
}
