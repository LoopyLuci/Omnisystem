//! Backup/restore: validates a chain of full and incremental backups, plans
//! the ordered sequence of records needed to restore to a given point, and
//! applies a "keep N full backups" retention policy without ever pruning a
//! full backup that an incremental still depends on.

#![warn(missing_docs)]

use std::collections::{HashMap, HashSet};

pub mod error;
pub mod types;

pub use error::{Error, Result};
pub use types::*;

fn index(records: &[BackupRecord]) -> Result<HashMap<&str, &BackupRecord>> {
    let mut by_id = HashMap::new();
    for r in records {
        if by_id.insert(r.id.as_str(), r).is_some() {
            return Err(Error::DuplicateId(r.id.clone()));
        }
    }
    Ok(by_id)
}

/// Validate that every incremental backup's parent exists and that the
/// chain from every backup terminates at a full backup (no dangling or
/// cyclic parent references).
pub fn validate_chains(records: &[BackupRecord]) -> Result<()> {
    let by_id = index(records)?;
    for r in records {
        let mut current = r;
        let mut visited = HashSet::new();
        loop {
            if !visited.insert(current.id.as_str()) {
                return Err(Error::BrokenChain(r.id.clone()));
            }
            match &current.kind {
                BackupKind::Full => break,
                BackupKind::Incremental { parent } => match by_id.get(parent.as_str()) {
                    Some(p) => current = p,
                    None => {
                        return Err(Error::MissingParent { id: current.id.clone(), parent: parent.clone() })
                    }
                },
            }
        }
    }
    Ok(())
}

/// Compute the ordered list of backup ids (oldest/base first) needed to
/// restore `target_id`, from the base full backup through to the target.
pub fn restore_plan(records: &[BackupRecord], target_id: &str) -> Result<Vec<String>> {
    validate_chains(records)?;
    let by_id = index(records)?;
    let mut current = *by_id.get(target_id).ok_or_else(|| Error::NotFound(target_id.to_string()))?;
    let mut chain = vec![current.id.clone()];
    while let BackupKind::Incremental { parent } = &current.kind {
        current = by_id[parent.as_str()];
        chain.push(current.id.clone());
    }
    chain.reverse();
    Ok(chain)
}

/// Apply a retention policy: keep the `keep_full` most-recent full backups
/// (by `day`, descending) and every incremental whose chain depends on a
/// kept full backup; return the ids to prune.
pub fn plan_pruning(records: &[BackupRecord], policy: RetentionPolicy) -> Result<Vec<String>> {
    validate_chains(records)?;
    let mut fulls: Vec<&BackupRecord> =
        records.iter().filter(|r| matches!(r.kind, BackupKind::Full)).collect();
    fulls.sort_by(|a, b| b.day.cmp(&a.day));
    let kept_fulls: HashSet<&str> = fulls.iter().take(policy.keep_full).map(|r| r.id.as_str()).collect();

    let by_id = index(records)?;
    let mut keep: HashSet<String> = HashSet::new();
    for r in records {
        let mut current = r;
        loop {
            match &current.kind {
                BackupKind::Full => {
                    if kept_fulls.contains(current.id.as_str()) {
                        keep.insert(r.id.clone());
                    }
                    break;
                }
                BackupKind::Incremental { parent } => current = by_id[parent.as_str()],
            }
        }
    }

    let mut prune: Vec<String> = records.iter().map(|r| r.id.clone()).filter(|id| !keep.contains(id)).collect();
    prune.sort();
    Ok(prune)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn full(id: &str, day: i64) -> BackupRecord {
        BackupRecord { id: id.into(), day, kind: BackupKind::Full }
    }
    fn incr(id: &str, day: i64, parent: &str) -> BackupRecord {
        BackupRecord { id: id.into(), day, kind: BackupKind::Incremental { parent: parent.into() } }
    }

    #[test]
    fn validate_rejects_duplicate_ids() {
        let records = vec![full("a", 1), full("a", 2)];
        assert_eq!(validate_chains(&records), Err(Error::DuplicateId("a".into())));
    }

    #[test]
    fn validate_rejects_missing_parent() {
        let records = vec![incr("b", 2, "ghost")];
        assert!(matches!(validate_chains(&records), Err(Error::MissingParent { .. })));
    }

    #[test]
    fn validate_rejects_cyclic_chain() {
        let records = vec![incr("a", 1, "b"), incr("b", 2, "a")];
        assert!(matches!(validate_chains(&records), Err(Error::BrokenChain(_))));
    }

    #[test]
    fn validate_accepts_well_formed_chain() {
        let records = vec![full("f1", 1), incr("i1", 2, "f1"), incr("i2", 3, "i1")];
        assert!(validate_chains(&records).is_ok());
    }

    #[test]
    fn restore_plan_orders_base_first() {
        let records = vec![full("f1", 1), incr("i1", 2, "f1"), incr("i2", 3, "i1")];
        let plan = restore_plan(&records, "i2").unwrap();
        assert_eq!(plan, vec!["f1".to_string(), "i1".to_string(), "i2".to_string()]);
    }

    #[test]
    fn restore_plan_of_full_backup_is_itself() {
        let records = vec![full("f1", 1)];
        assert_eq!(restore_plan(&records, "f1").unwrap(), vec!["f1".to_string()]);
    }

    #[test]
    fn restore_plan_errors_on_unknown_target() {
        let records = vec![full("f1", 1)];
        assert_eq!(restore_plan(&records, "ghost"), Err(Error::NotFound("ghost".into())));
    }

    #[test]
    fn plan_pruning_keeps_most_recent_fulls_and_their_incrementals() {
        let records = vec![
            full("f1", 1),
            incr("i1", 2, "f1"),
            full("f2", 3),
            incr("i2", 4, "f2"),
            full("f3", 5),
        ];
        let prune = plan_pruning(&records, RetentionPolicy { keep_full: 2 }).unwrap();
        // Oldest full (f1, day 1) and its dependent incremental (i1) are pruned;
        // f2/i2 and f3 are kept as the 2 most recent full chains.
        assert_eq!(prune, vec!["f1".to_string(), "i1".to_string()]);
    }

    #[test]
    fn plan_pruning_keep_all_prunes_nothing() {
        let records = vec![full("f1", 1), full("f2", 2)];
        let prune = plan_pruning(&records, RetentionPolicy { keep_full: 5 }).unwrap();
        assert!(prune.is_empty());
    }
}
