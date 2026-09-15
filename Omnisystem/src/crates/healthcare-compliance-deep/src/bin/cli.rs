//! Demo CLI: evaluate one access event and one breach incident.

use healthcare_compliance_deep::{
    AccessEvent, AccessPurpose, BreachIncident, ComplianceEngine, ConsentScope, PhiCategory,
};

fn main() {
    let mut engine = ComplianceEngine::new();
    engine.register_consent(
        ConsentScope::new("patient-001")
            .allow_purpose(AccessPurpose::Treatment)
            .allow_category(PhiCategory::Diagnosis),
    );

    let event = AccessEvent {
        patient_id: "patient-001".into(),
        purpose: AccessPurpose::Treatment,
        categories_accessed: vec![PhiCategory::Diagnosis, PhiCategory::MentalHealth],
    };
    println!("access decision: {:?}", engine.evaluate_access(&event));

    let incident = BreachIncident {
        categories: vec![PhiCategory::Genetic],
        affected_individuals: 1200,
        encrypted: false,
        acquired_or_viewed: true,
    };
    println!("breach risk: {:?}", engine.assess_breach(&incident));
}
