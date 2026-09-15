//! Demo CLI: compute a restore plan for an incremental backup chain.

use backup_restore_ui::{restore_plan, BackupKind, BackupRecord};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let records = vec![
        BackupRecord { id: "full-2026-07-01".into(), day: 1, kind: BackupKind::Full },
        BackupRecord {
            id: "incr-2026-07-08".into(),
            day: 8,
            kind: BackupKind::Incremental { parent: "full-2026-07-01".into() },
        },
        BackupRecord {
            id: "incr-2026-07-13".into(),
            day: 13,
            kind: BackupKind::Incremental { parent: "incr-2026-07-08".into() },
        },
    ];
    let plan = restore_plan(&records, "incr-2026-07-13")?;
    println!("restore plan (apply in order): {}", plan.join(" -> "));
    Ok(())
}
