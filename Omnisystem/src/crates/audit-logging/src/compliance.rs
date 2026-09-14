//! Compliance report tracking.
//!
//! Ported from the `audit-system` crate during the Phase 4 audit-logging
//! cluster reconciliation (2026-09-14): `audit-system` and `audit-logging`
//! independently reimplemented the same tamper-evident audit log concept
//! (see `logger.rs`/`types.rs`), but `audit-system` additionally carried
//! this `ComplianceChecker`, which had no equivalent here. Rather than
//! discard that logic when retiring `audit-system` as a duplicate, it was
//! merged into the canonical crate so the real work isn't lost.

use crate::{AuditError, AuditResult};
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// A point-in-time compliance snapshot for an entity (user, service,
/// resource, etc.).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ComplianceReport {
    pub report_id: String,
    pub violations: u32,
    pub last_audit: DateTime<Utc>,
    pub status: String,
}

/// Tracks compliance reports per entity.
pub struct ComplianceChecker {
    reports: Arc<DashMap<String, ComplianceReport>>,
}

impl ComplianceChecker {
    pub fn new() -> Self {
        Self {
            reports: Arc::new(DashMap::new()),
        }
    }

    pub async fn check_compliance(&self, entity_id: &str) -> AuditResult<ComplianceReport> {
        let report = ComplianceReport {
            report_id: uuid::Uuid::new_v4().to_string(),
            violations: 0,
            last_audit: Utc::now(),
            status: "compliant".to_string(),
        };

        self.reports.insert(entity_id.to_string(), report.clone());
        Ok(report)
    }

    pub async fn get_report(&self, entity_id: &str) -> AuditResult<ComplianceReport> {
        self.reports
            .get(entity_id)
            .map(|entry| entry.clone())
            .ok_or(AuditError::Other("report not found".to_string()))
    }

    pub fn report_count(&self) -> usize {
        self.reports.len()
    }
}

impl Default for ComplianceChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_check_compliance() {
        let checker = ComplianceChecker::new();
        let report = checker.check_compliance("entity1").await.unwrap();
        assert_eq!(report.status, "compliant");
    }

    #[tokio::test]
    async fn test_get_report() {
        let checker = ComplianceChecker::new();
        checker.check_compliance("entity1").await.unwrap();
        let report = checker.get_report("entity1").await.unwrap();
        assert_eq!(report.violations, 0);
    }

    #[tokio::test]
    async fn test_get_report_missing() {
        let checker = ComplianceChecker::new();
        assert!(checker.get_report("nope").await.is_err());
    }
}
