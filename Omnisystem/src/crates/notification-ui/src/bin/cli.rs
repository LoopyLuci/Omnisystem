//! Demo CLI: dedupe a small notification queue and print unread counts.

use notification_ui::{dedupe, unread_counts_by_category, Notification, Priority};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let notifications = vec![
        Notification {
            id: "n1".into(),
            category: "billing".into(),
            priority: Priority::High,
            seq: 1,
            dedup_key: Some("invoice-overdue".into()),
            read: false,
        },
        Notification {
            id: "n2".into(),
            category: "billing".into(),
            priority: Priority::High,
            seq: 2,
            dedup_key: Some("invoice-overdue".into()),
            read: false,
        },
    ];
    let deduped = dedupe(&notifications, 5)?;
    println!("after dedupe: {} notification(s)", deduped.len());
    println!("unread by category: {:?}", unread_counts_by_category(&deduped));
    Ok(())
}
