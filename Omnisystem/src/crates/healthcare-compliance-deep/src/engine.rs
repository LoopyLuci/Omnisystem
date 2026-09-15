//! Consent-scope registry, minimum-necessary access evaluation, and
//! breach-risk assessment.

use crate::error::{Error, Result};
use crate::types::{AccessDecision, AccessEvent, BreachIncident, BreachRiskLevel, ConsentScope};
use std::collections::BTreeMap;

/// Registry of per-patient consent scopes, and the engine that evaluates
/// access events and breach incidents against them.
#[derive(Debug, Default)]
pub struct ComplianceEngine {
    consents: BTreeMap<String, ConsentScope>,
}

impl ComplianceEngine {
    /// Create an empty engine (no consent on file for anyone).
    pub fn new() -> Self {
        Self::default()
    }

    /// Record (or replace) a patient's consent scope.
    pub fn register_consent(&mut self, scope: ConsentScope) {
        self.consents.insert(scope.patient_id.clone(), scope);
    }

    /// Look up a patient's consent scope.
    pub fn consent_for(&self, patient_id: &str) -> Result<&ConsentScope> {
        self.consents.get(patient_id).ok_or_else(|| Error::NotFound(patient_id.to_string()))
    }

    /// Evaluate an access event against the patient's consent on file,
    /// enforcing both purpose authorization and the "minimum necessary"
    /// rule (every category actually accessed must be individually
    /// consented, not just the purpose in general).
    pub fn evaluate_access(&self, event: &AccessEvent) -> AccessDecision {
        let Some(scope) = self.consents.get(&event.patient_id) else {
            return AccessDecision::NoConsentOnFile;
        };
        if !scope.allowed_purposes.contains(&event.purpose) {
            return AccessDecision::PurposeNotConsented;
        }
        let unauthorized: Vec<_> = event
            .categories_accessed
            .iter()
            .copied()
            .filter(|c| !scope.allowed_categories.contains(c))
            .collect();
        if unauthorized.is_empty() {
            AccessDecision::Permitted
        } else {
            AccessDecision::MinimumNecessaryViolation { unauthorized_categories: unauthorized }
        }
    }

    /// Assess breach risk for an incident per the encryption/no-access
    /// safe harbor, then a sensitivity/scale-weighted score.
    pub fn assess_breach(&self, incident: &BreachIncident) -> BreachRiskLevel {
        if incident.encrypted || !incident.acquired_or_viewed {
            return BreachRiskLevel::SafeHarbor;
        }
        let severity_score: u32 = incident.categories.iter().map(|c| c.sensitivity_weight()).sum();
        if severity_score >= 15 || incident.affected_individuals >= 500 {
            BreachRiskLevel::High
        } else if severity_score >= 8 || incident.affected_individuals >= 50 {
            BreachRiskLevel::Moderate
        } else {
            BreachRiskLevel::Low
        }
    }
}
