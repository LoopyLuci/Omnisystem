//! security-console-ui: dashboard data-shaping for the security cluster.
//!
//! Has no security-detection logic of its own — it presents data produced
//! by the other crates in this cluster (`omnisystem-security-integration`,
//! `container-security-platform`, `security-analyzer`, etc.), modeled here
//! generically as [`types::SourceEvent`]. Its real job is shaping: turning
//! a flat event list into a severity-histogram [`types::DashboardSummary`]
//! ([`shaper::summarize`]) and a sorted, paginated [`types::EventPage`]
//! ([`shaper::paginate`]) — the two views an actual console UI renders.

pub mod error;
pub mod shaper;
pub mod types;

pub use error::{Error, Result};
pub use shaper::{filter_by_source, paginate, summarize};
pub use types::*;

#[cfg(test)]
mod tests {
    use super::*;

    fn event(id: &str, source: &str, severity: u8) -> SourceEvent {
        SourceEvent { id: id.to_string(), source: source.to_string(), severity, title: format!("event {id}") }
    }

    #[test]
    fn summarize_empty_has_zeroed_buckets() {
        let summary = summarize(&[]);
        assert_eq!(summary.severity_buckets.len(), 5);
        assert!(summary.severity_buckets.iter().all(|b| b.count == 0));
        assert_eq!(summary.total_events, 0);
    }

    #[test]
    fn summarize_counts_by_severity() {
        let events = vec![event("1", "a", 4), event("2", "a", 4), event("3", "b", 1)];
        let summary = summarize(&events);
        assert_eq!(summary.severity_buckets[4].count, 2);
        assert_eq!(summary.severity_buckets[1].count, 1);
        assert_eq!(summary.total_events, 3);
    }

    #[test]
    fn summarize_collects_distinct_sorted_sources() {
        let events = vec![event("1", "zeta", 0), event("2", "alpha", 0), event("3", "alpha", 1)];
        let summary = summarize(&events);
        assert_eq!(summary.sources, vec!["alpha".to_string(), "zeta".to_string()]);
    }

    #[test]
    fn severity_clamped_above_four() {
        let events = vec![event("1", "a", 99)];
        let summary = summarize(&events);
        assert_eq!(summary.severity_buckets[4].count, 1);
    }

    #[test]
    fn paginate_sorts_highest_severity_first() {
        let events = vec![event("1", "a", 1), event("2", "a", 4), event("3", "a", 2)];
        let page = paginate(&events, 0, 10);
        assert_eq!(page.items.iter().map(|e| e.id.clone()).collect::<Vec<_>>(), vec!["2", "3", "1"]);
        assert_eq!(page.total, 3);
    }

    #[test]
    fn paginate_ties_broken_by_id_for_determinism() {
        let events = vec![event("b", "a", 3), event("a", "a", 3)];
        let page = paginate(&events, 0, 10);
        assert_eq!(page.items[0].id, "a");
        assert_eq!(page.items[1].id, "b");
    }

    #[test]
    fn paginate_returns_correct_slice() {
        let events: Vec<_> = (0..25).map(|i| event(&i.to_string(), "a", 0)).collect();
        let page = paginate(&events, 1, 10);
        assert_eq!(page.items.len(), 10);
        assert_eq!(page.page, 1);
        assert_eq!(page.total, 25);
    }

    #[test]
    fn paginate_past_end_returns_empty() {
        let events = vec![event("1", "a", 0)];
        let page = paginate(&events, 5, 10);
        assert!(page.items.is_empty());
        assert_eq!(page.total, 1);
    }

    #[test]
    fn filter_by_source_only_returns_matching() {
        let events = vec![event("1", "container-security-platform", 3), event("2", "security-analyzer", 2)];
        let filtered = filter_by_source(&events, "security-analyzer");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "2");
    }
}
