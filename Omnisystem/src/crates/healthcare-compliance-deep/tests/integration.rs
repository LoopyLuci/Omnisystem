//! Integration test: consented treatment access alongside an unrelated
//! minimum-necessary violation and a breach assessment, all against one
//! shared engine.

use healthcare_compliance_deep::{
    AccessDecision, AccessEvent, AccessPurpose, BreachIncident, BreachRiskLevel, ComplianceEngine, ConsentScope,
    PhiCategory,
};

#[test]
fn clinic_scenario_end_to_end() {
    let mut engine = ComplianceEngine::new();
    engine.register_consent(
        ConsentScope::new("patient-alpha")
            .allow_purpose(AccessPurpose::Treatment)
            .allow_purpose(AccessPurpose::Payment)
            .allow_category(PhiCategory::Diagnosis)
            .allow_category(PhiCategory::Treatment)
            .allow_category(PhiCategory::Billing),
    );

    // A billing clerk reading only billing data for payment: fine.
    let billing_access = AccessEvent {
        patient_id: "patient-alpha".into(),
        purpose: AccessPurpose::Payment,
        categories_accessed: vec![PhiCategory::Billing],
    };
    assert!(engine.evaluate_access(&billing_access).is_permitted());

    // A researcher trying to pull the same record without consent for
    // research: purpose not consented.
    let research_access = AccessEvent {
        patient_id: "patient-alpha".into(),
        purpose: AccessPurpose::Research,
        categories_accessed: vec![PhiCategory::Diagnosis],
    };
    assert_eq!(engine.evaluate_access(&research_access), AccessDecision::PurposeNotConsented);

    // A treating clinician who also pulls mental-health notes that were
    // never authorized: minimum-necessary violation, even though the
    // purpose itself (Treatment) is consented.
    let overreaching_access = AccessEvent {
        patient_id: "patient-alpha".into(),
        purpose: AccessPurpose::Treatment,
        categories_accessed: vec![PhiCategory::Diagnosis, PhiCategory::MentalHealth],
    };
    match engine.evaluate_access(&overreaching_access) {
        AccessDecision::MinimumNecessaryViolation { unauthorized_categories } => {
            assert_eq!(unauthorized_categories, vec![PhiCategory::MentalHealth]);
        }
        other => panic!("expected minimum-necessary violation, got {other:?}"),
    }

    // Separately: a lost unencrypted laptop with a large mixed-sensitivity
    // export is assessed as high risk and would trigger notification.
    let incident = BreachIncident {
        categories: vec![PhiCategory::Diagnosis, PhiCategory::SubstanceAbuse],
        affected_individuals: 600,
        encrypted: false,
        acquired_or_viewed: true,
    };
    assert_eq!(engine.assess_breach(&incident), BreachRiskLevel::High);
}
