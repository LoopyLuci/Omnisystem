//! Clinical-data HIPAA compliance domain types: PHI categories, consent
//! scopes, access events, and breach risk.
//!
//! `healthcare-compliance-deep` models HIPAA compliance at the level of
//! individual clinical data accesses and breach-risk assessment (the
//! "deep", per-PHI-category, per-access-event angle). It is distinct in
//! scope from `medical-compliance`, which — once inspected — turned out to
//! be a byte-for-byte identical scaffold with no distinguishing logic; see
//! this crate's `lib.rs` doc comment and the phase-5 census report for the
//! dedup recommendation.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// A category of Protected Health Information, each carrying a relative
/// sensitivity weight used in breach-risk scoring. Categories subject to
/// extra federal protection (42 CFR Part 2 substance-use records, genetic
/// information under GINA, mental health notes) are weighted highest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PhiCategory {
    /// Name, address, DOB, contact info.
    Demographics,
    /// Insurance/payment records.
    Billing,
    /// Diagnosis codes and clinical notes.
    Diagnosis,
    /// Treatment/procedure records.
    Treatment,
    /// Genetic test results (GINA-protected).
    Genetic,
    /// Mental health treatment notes.
    MentalHealth,
    /// Substance use disorder records (42 CFR Part 2).
    SubstanceAbuse,
}

impl PhiCategory {
    /// Relative sensitivity weight (1 = least, 5 = most) used in breach
    /// risk scoring.
    pub fn sensitivity_weight(self) -> u32 {
        match self {
            PhiCategory::Demographics => 1,
            PhiCategory::Billing => 2,
            PhiCategory::Diagnosis => 3,
            PhiCategory::Treatment => 3,
            PhiCategory::Genetic => 5,
            PhiCategory::MentalHealth => 5,
            PhiCategory::SubstanceAbuse => 5,
        }
    }
}

/// The purpose an access to PHI is claimed to be for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AccessPurpose {
    /// Direct patient treatment.
    Treatment,
    /// Billing/payment processing.
    Payment,
    /// Internal quality/operations review.
    HealthcareOperations,
    /// Research use.
    Research,
    /// Marketing (requires explicit authorization in almost all cases).
    Marketing,
}

/// A patient's on-file consent: which purposes and PHI categories they
/// have authorized access to.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConsentScope {
    /// Patient identifier.
    pub patient_id: String,
    /// Purposes this patient has consented to.
    pub allowed_purposes: BTreeSet<AccessPurpose>,
    /// PHI categories this patient has consented to release.
    pub allowed_categories: BTreeSet<PhiCategory>,
}

impl ConsentScope {
    /// A new, empty consent scope for a patient (default-deny).
    pub fn new(patient_id: impl Into<String>) -> Self {
        Self { patient_id: patient_id.into(), allowed_purposes: BTreeSet::new(), allowed_categories: BTreeSet::new() }
    }

    /// Authorize a purpose (builder-style).
    pub fn allow_purpose(mut self, purpose: AccessPurpose) -> Self {
        self.allowed_purposes.insert(purpose);
        self
    }

    /// Authorize a PHI category (builder-style).
    pub fn allow_category(mut self, category: PhiCategory) -> Self {
        self.allowed_categories.insert(category);
        self
    }
}

/// A single access request/event against a patient's record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessEvent {
    /// Patient whose record was accessed.
    pub patient_id: String,
    /// Claimed purpose of the access.
    pub purpose: AccessPurpose,
    /// PHI categories actually read in this access.
    pub categories_accessed: Vec<PhiCategory>,
}

/// The result of evaluating an access event against consent on file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccessDecision {
    /// Access is within consented purpose and categories.
    Permitted,
    /// No consent scope exists for this patient at all.
    NoConsentOnFile,
    /// Consent exists but does not cover the claimed purpose.
    PurposeNotConsented,
    /// Consent exists and covers the purpose, but one or more accessed
    /// categories were not authorized (a "minimum necessary" violation:
    /// more categories were read than the purpose was consented for).
    MinimumNecessaryViolation { unauthorized_categories: Vec<PhiCategory> },
}

impl AccessDecision {
    /// True if the access was permitted outright.
    pub fn is_permitted(&self) -> bool {
        matches!(self, AccessDecision::Permitted)
    }
}

/// HIPAA breach risk level, per the four-factor risk assessment in the
/// 2013 Omnibus Rule (45 CFR 164.402): nature/extent of PHI, who
/// acquired/viewed it, whether it was actually acquired or viewed, and
/// mitigation. This model collapses that into a deterministic score from
/// PHI-category sensitivity and affected-individual count, with an
/// encryption safe harbor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BreachRiskLevel {
    /// Encrypted per NIST standard, or PHI confirmed never acquired/viewed:
    /// no breach notification obligation (encryption/no-access safe harbor).
    SafeHarbor,
    /// Low probability of compromise.
    Low,
    /// Moderate probability of compromise.
    Moderate,
    /// High probability of compromise; notification obligations apply.
    High,
}

/// Inputs to a breach risk assessment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreachIncident {
    /// PHI categories exposed in the incident.
    pub categories: Vec<PhiCategory>,
    /// Number of individuals whose records were affected.
    pub affected_individuals: usize,
    /// Whether the exposed data was encrypted to NIST standard.
    pub encrypted: bool,
    /// Whether the PHI was confirmed actually acquired or viewed by an
    /// unauthorized party (vs. e.g. a device recovered unopened).
    pub acquired_or_viewed: bool,
}
