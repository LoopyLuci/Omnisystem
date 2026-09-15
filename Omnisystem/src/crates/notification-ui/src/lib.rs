//! Notification UI: dedups notifications sharing a `dedup_key` within a
//! sequence window (keeping the most recent), sorts by priority then
//! recency for display, and summarizes unread counts by category.

#![warn(missing_docs)]

use std::collections::BTreeMap;

pub mod error;
pub mod types;

pub use error::{Error, Result};
pub use types::*;

fn validate(notifications: &[Notification]) -> Result<()> {
    let mut seen = std::collections::HashSet::new();
    for n in notifications {
        if !seen.insert(n.id.clone()) {
            return Err(Error::DuplicateId(n.id.clone()));
        }
    }
    Ok(())
}

/// Collapse notifications that share a `dedup_key` and fall within `window`
/// sequence numbers of the most recent one for that key (i.e. `latest_seq -
/// seq <= window`), keeping only the most recent. Notifications with no
/// `dedup_key`, or whose key's latest occurrence is more than `window`
/// away, are never collapsed.
pub fn dedupe(notifications: &[Notification], window: i64) -> Result<Vec<Notification>> {
    validate(notifications)?;
    let mut latest_by_key: BTreeMap<&str, i64> = BTreeMap::new();
    for n in notifications {
        if let Some(key) = &n.dedup_key {
            let entry = latest_by_key.entry(key.as_str()).or_insert(n.seq);
            if n.seq > *entry {
                *entry = n.seq;
            }
        }
    }
    let mut out: Vec<Notification> = notifications
        .iter()
        .filter(|n| match &n.dedup_key {
            None => true,
            Some(key) => {
                let latest = latest_by_key[key.as_str()];
                n.seq == latest || latest - n.seq > window
            }
        })
        .cloned()
        .collect();
    out.sort_by_key(|n| n.seq);
    Ok(out)
}

/// Sort for display: highest priority first, then most recent first.
pub fn sort_for_display(mut notifications: Vec<Notification>) -> Vec<Notification> {
    notifications.sort_by(|a, b| b.priority.cmp(&a.priority).then(b.seq.cmp(&a.seq)));
    notifications
}

/// Count of unread notifications per category, sorted by category name.
pub fn unread_counts_by_category(notifications: &[Notification]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for n in notifications.iter().filter(|n| !n.read) {
        *counts.entry(n.category.clone()).or_insert(0usize) += 1;
    }
    counts
}

#[cfg(test)]
mod tests {
    use super::*;

    fn notif(id: &str, seq: i64, key: Option<&str>) -> Notification {
        Notification {
            id: id.into(),
            category: "general".into(),
            priority: Priority::Normal,
            seq,
            dedup_key: key.map(|k| k.to_string()),
            read: false,
        }
    }

    #[test]
    fn validate_rejects_duplicate_ids_via_dedupe() {
        let notifications = vec![notif("a", 1, None), notif("a", 2, None)];
        assert!(matches!(dedupe(&notifications, 10), Err(Error::DuplicateId(id)) if id == "a"));
    }

    #[test]
    fn dedupe_collapses_within_window_keeping_latest() {
        let notifications = vec![notif("a", 1, Some("k")), notif("b", 2, Some("k"))];
        let result = dedupe(&notifications, 5).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, "b");
    }

    #[test]
    fn dedupe_keeps_both_outside_window() {
        let notifications = vec![notif("a", 1, Some("k")), notif("b", 20, Some("k"))];
        let result = dedupe(&notifications, 5).unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn dedupe_never_collapses_notifications_without_a_key() {
        let notifications = vec![notif("a", 1, None), notif("b", 2, None)];
        let result = dedupe(&notifications, 100).unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn sort_for_display_orders_by_priority_then_recency() {
        let mut low = notif("a", 5, None);
        low.priority = Priority::Low;
        let mut high = notif("b", 1, None);
        high.priority = Priority::High;
        let normal = notif("c", 10, None);
        let sorted = sort_for_display(vec![low, normal.clone(), high]);
        assert_eq!(sorted[0].id, "b"); // High wins regardless of seq
        assert_eq!(sorted[1].id, "c"); // Normal, most recent
        assert_eq!(sorted[2].id, "a"); // Low
    }

    #[test]
    fn unread_counts_by_category_ignores_read() {
        let mut n1 = notif("a", 1, None);
        n1.category = "billing".into();
        let mut n2 = notif("b", 2, None);
        n2.category = "billing".into();
        n2.read = true;
        let counts = unread_counts_by_category(&[n1, n2]);
        assert_eq!(counts.get("billing"), Some(&1));
    }

    #[test]
    fn unread_counts_groups_multiple_categories() {
        let mut n1 = notif("a", 1, None);
        n1.category = "billing".into();
        let mut n2 = notif("b", 2, None);
        n2.category = "security".into();
        let counts = unread_counts_by_category(&[n1, n2]);
        assert_eq!(counts.len(), 2);
    }

    #[test]
    fn dedupe_preserves_chronological_order_of_survivors() {
        let notifications = vec![notif("a", 5, None), notif("b", 1, None)];
        let result = dedupe(&notifications, 10).unwrap();
        assert_eq!(result[0].id, "b");
        assert_eq!(result[1].id, "a");
    }
}
