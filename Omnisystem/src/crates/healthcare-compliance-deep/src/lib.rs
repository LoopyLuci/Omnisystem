//! healthcare-compliance-deep: clinical-data-level HIPAA compliance.
//!
//! Models two real, testable HIPAA concerns at the individual-access and
//! individual-incident level (the "deep" angle, as opposed to org-wide
//! policy checklists):
//!
//! 1. **Minimum-necessary access evaluation** ([`engine::ComplianceEngine::evaluate_access`]):
//!    given a patient's on-file [`types::ConsentScope`] (authorized
//!    purposes + PHI categories) and an [`types::AccessEvent`], decide
//!    whether the access was permitted, purpose-unauthorized, or a
//!    minimum-necessary violation (categories read beyond what was
//!    consented).
//! 2. **Breach risk assessment** ([`engine::ComplianceEngine::assess_breach`]):
//!    given a [`types::BreachIncident`] (PHI categories exposed, affected
//!    individual count, encryption status, acquired/viewed confirmation),
//!    compute a [`types::BreachRiskLevel`] using the encryption/no-access
//!    safe harbor from the 2013 HIPAA Omnibus Rule plus a
//!    sensitivity-weighted severity score.
//!
//! ## Relationship to `medical-compliance`
//!
//! Before writing any of this, `medical-compliance`'s source tree was
//! diffed against this crate's pre-build-out scaffold: they were
//! byte-for-byte identical (only the CLI binary name differed) — the same
//! generic `Record`/`Manager` CRUD scaffold with zero clinical-data-specific
//! logic in either. Rather than build the same domain twice under two
//! names, this crate was built out with real logic and `medical-compliance`
//! was left as a flagged dedup candidate (see the phase-5 census report) —
//! recommend reconciling it into this crate the way `audit-system` was
//! reconciled into `audit-logging` in Phase 4, in a future session with its
//! own reverse-dependency care.

pub mod engine;
pub mod error;
pub mod types;

pub use engine::ComplianceEngine;
pub use error::{Error, Result};
pub use types::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_consent_on_file_denies_access() {
        let engine = ComplianceEngine::new();
        let event = AccessEvent {
            patient_id: "p1".into(),
            purpose: AccessPurpose::Treatment,
            categories_accessed: vec![PhiCategory::Diagnosis],
        };
        assert_eq!(engine.evaluate_access(&event), AccessDecision::NoConsentOnFile);
    }

    #[test]
    fn purpose_not_consented_is_denied() {
        let mut engine = ComplianceEngine::new();
        engine.register_consent(ConsentScope::new("p1").allow_purpose(AccessPurpose::Treatment));
        let event = AccessEvent {
            patient_id: "p1".into(),
            purpose: AccessPurpose::Marketing,
            categories_accessed: vec![],
        };
        assert_eq!(engine.evaluate_access(&event), AccessDecision::PurposeNotConsented);
    }

    #[test]
    fn fully_consented_access_is_permitted() {
        let mut engine = ComplianceEngine::new();
        engine.register_consent(
            ConsentScope::new("p1")
                .allow_purpose(AccessPurpose::Treatment)
                .allow_category(PhiCategory::Diagnosis)
                .allow_category(PhiCategory::Treatment),
        );
        let event = AccessEvent {
            patient_id: "p1".into(),
            purpose: AccessPurpose::Treatment,
            categories_accessed: vec![PhiCategory::Diagnosis],
        };
        assert!(engine.evaluate_access(&event).is_permitted());
    }

    #[test]
    fn reading_uncosented_category_is_minimum_necessary_violation() {
        let mut engine = ComplianceEngine::new();
        engine.register_consent(
            ConsentScope::new("p1").allow_purpose(AccessPurpose::Treatment).allow_category(PhiCategory::Diagnosis),
        );
        let event = AccessEvent {
            patient_id: "p1".into(),
            purpose: AccessPurpose::Treatment,
            categories_accessed: vec![PhiCategory::Diagnosis, PhiCategory::MentalHealth],
        };
        match engine.evaluate_access(&event) {
            AccessDecision::MinimumNecessaryViolation { unauthorized_categories } => {
                assert_eq!(unauthorized_categories, vec![PhiCategory::MentalHealth]);
            }
            other => panic!("expected violation, got {other:?}"),
        }
    }

    #[test]
    fn encrypted_incident_is_safe_harbor() {
        let engine = ComplianceEngine::new();
        let incident = BreachIncident {
            categories: vec![PhiCategory::Genetic],
            affected_individuals: 10_000,
            encrypted: true,
            acquired_or_viewed: true,
        };
        assert_eq!(engine.assess_breach(&incident), BreachRiskLevel::SafeHarbor);
    }

    #[test]
    fn unencrypted_but_never_viewed_is_safe_harbor() {
        let engine = ComplianceEngine::new();
        let incident = BreachIncident {
            categories: vec![PhiCategory::Genetic],
            affected_individuals: 500,
            encrypted: false,
            acquired_or_viewed: false,
        };
        assert_eq!(engine.assess_breach(&incident), BreachRiskLevel::SafeHarbor);
    }

    #[test]
    fn small_low_sensitivity_exposure_is_low_risk() {
        let engine = ComplianceEngine::new();
        let incident = BreachIncident {
            categories: vec![PhiCategory::Demographics],
            affected_individuals: 3,
            encrypted: false,
            acquired_or_viewed: true,
        };
        assert_eq!(engine.assess_breach(&incident), BreachRiskLevel::Low);
    }

    #[test]
    fn large_scale_unencrypted_exposure_is_high_risk() {
        let engine = ComplianceEngine::new();
        let incident = BreachIncident {
            categories: vec![PhiCategory::SubstanceAbuse, PhiCategory::Genetic],
            affected_individuals: 800,
            encrypted: false,
            acquired_or_viewed: true,
        };
        assert_eq!(engine.assess_breach(&incident), BreachRiskLevel::High);
    }

    #[test]
    fn moderate_sensitivity_small_scale_is_moderate() {
        let engine = ComplianceEngine::new();
        let incident = BreachIncident {
            categories: vec![PhiCategory::Diagnosis, PhiCategory::Treatment, PhiCategory::Billing],
            affected_individuals: 20,
            encrypted: false,
            acquired_or_viewed: true,
        };
        assert_eq!(engine.assess_breach(&incident), BreachRiskLevel::Moderate);
    }

    #[test]
    fn unknown_patient_consent_lookup_errors() {
        let engine = ComplianceEngine::new();
        assert!(matches!(engine.consent_for("ghost"), Err(Error::NotFound(_))));
    }
}
