//! Evaluates submitted evidence against defined controls to produce a
//! compliance report.

use crate::error::{Error, Result};
use crate::types::{ComplianceReport, Control, ControlResult, ControlStatus, Evidence};
use std::collections::BTreeMap;

/// Holds control definitions and evaluates evidence against them.
#[derive(Debug, Default)]
pub struct ComplianceEngine {
    controls: BTreeMap<String, Control>,
}

impl ComplianceEngine {
    /// Create an empty engine.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a control definition.
    pub fn define_control(&mut self, control: Control) {
        self.controls.insert(control.id.clone(), control);
    }

    /// Evaluate a batch of evidence (which may span multiple controls and
    /// include duplicate/conflicting entries for the same evidence type —
    /// the last valid entry for a type wins) against every defined control,
    /// producing one result per control, in definition order.
    pub fn evaluate(&self, evidence: &[Evidence]) -> ComplianceReport {
        let mut results = Vec::new();
        for control in self.controls.values() {
            let mut satisfied = Vec::new();
            let mut missing = Vec::new();
            for evidence_type in &control.required_evidence {
                let has_valid = evidence.iter().any(|e| {
                    e.control_id == control.id && &e.evidence_type == evidence_type && e.valid
                });
                if has_valid {
                    satisfied.push(evidence_type.clone());
                } else {
                    missing.push(evidence_type.clone());
                }
            }
            let status = if control.required_evidence.is_empty() || missing.is_empty() {
                ControlStatus::Pass
            } else if satisfied.is_empty() {
                ControlStatus::Fail
            } else {
                ControlStatus::Partial
            };
            results.push(ControlResult {
                control_id: control.id.clone(),
                status,
                satisfied_evidence: satisfied,
                missing_evidence: missing,
            });
        }
        ComplianceReport { results }
    }

    /// Evaluate a single named control (error if it isn't defined).
    pub fn evaluate_control(&self, control_id: &str, evidence: &[Evidence]) -> Result<ControlResult> {
        if !self.controls.contains_key(control_id) {
            return Err(Error::UnknownControl(control_id.to_string()));
        }
        let report = self.evaluate(evidence);
        Ok(report
            .results
            .into_iter()
            .find(|r| r.control_id == control_id)
            .expect("control was just verified to exist"))
    }
}
