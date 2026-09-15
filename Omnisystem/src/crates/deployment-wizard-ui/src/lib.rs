//! Deployment wizard: walks a fixed sequence of [`WizardStep`]s, refusing
//! to advance while the current step's required answer fields are missing
//! or blank, and reporting completion progress.

#![warn(missing_docs)]

use std::collections::HashMap;

pub mod error;
pub mod types;

pub use error::{Error, Result};
pub use types::*;

/// Runtime state of an in-progress wizard.
#[derive(Debug, Clone)]
pub struct WizardState {
    definition: WizardDefinition,
    current: usize,
    answers: HashMap<String, String>,
}

impl WizardState {
    /// Start a wizard at its first step. Errors if the definition has no steps.
    pub fn new(definition: WizardDefinition) -> Result<Self> {
        if definition.steps.is_empty() {
            return Err(Error::NoSteps);
        }
        Ok(Self { definition, current: 0, answers: HashMap::new() })
    }

    /// The step currently being shown.
    pub fn current_step(&self) -> &WizardStep {
        &self.definition.steps[self.current]
    }

    /// Zero-based index of the current step.
    pub fn current_index(&self) -> usize {
        self.current
    }

    /// Record an answer for the current (or any) field.
    pub fn set_answer(&mut self, key: &str, value: &str) {
        self.answers.insert(key.to_string(), value.to_string());
    }

    /// Fields still missing (absent or empty) for the current step.
    pub fn missing_fields(&self) -> Vec<String> {
        self.current_step()
            .required_fields
            .iter()
            .filter(|f| self.answers.get(*f).map(|v| v.trim().is_empty()).unwrap_or(true))
            .cloned()
            .collect()
    }

    /// Whether the wizard can move to the next step from here.
    pub fn can_advance(&self) -> bool {
        self.missing_fields().is_empty()
    }

    /// Advance to the next step, validating the current step is complete
    /// first. Errors with [`Error::OutOfRange`] on the last step.
    pub fn advance(&mut self) -> Result<()> {
        let missing = self.missing_fields();
        if !missing.is_empty() {
            return Err(Error::IncompleteStep { step: self.current_step().name.clone(), missing });
        }
        if self.current + 1 >= self.definition.steps.len() {
            return Err(Error::OutOfRange);
        }
        self.current += 1;
        Ok(())
    }

    /// Go back to the previous step (no validation required).
    pub fn back(&mut self) -> Result<()> {
        if self.current == 0 {
            return Err(Error::OutOfRange);
        }
        self.current -= 1;
        Ok(())
    }

    /// Whether the wizard is on its final step.
    pub fn is_last_step(&self) -> bool {
        self.current + 1 == self.definition.steps.len()
    }

    /// Completion progress as a fraction of steps fully answered
    /// (`0.0..=1.0`), counting the current step only if it is complete.
    pub fn progress(&self) -> f64 {
        let total = self.definition.steps.len();
        let completed = (0..=self.current)
            .filter(|&i| {
                if i < self.current {
                    true
                } else {
                    self.can_advance()
                }
            })
            .count();
        completed as f64 / total as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wizard() -> WizardDefinition {
        WizardDefinition {
            steps: vec![
                WizardStep { name: "target".into(), required_fields: vec!["environment".into()] },
                WizardStep { name: "review".into(), required_fields: vec![] },
            ],
        }
    }

    #[test]
    fn new_errors_on_empty_definition() {
        assert!(matches!(WizardState::new(WizardDefinition::default()), Err(Error::NoSteps)));
    }

    #[test]
    fn missing_fields_reports_unset_required_field() {
        let state = WizardState::new(wizard()).unwrap();
        assert_eq!(state.missing_fields(), vec!["environment".to_string()]);
    }

    #[test]
    fn cannot_advance_with_missing_fields() {
        let mut state = WizardState::new(wizard()).unwrap();
        assert!(!state.can_advance());
        assert!(matches!(state.advance(), Err(Error::IncompleteStep { .. })));
    }

    #[test]
    fn advances_once_required_fields_set() {
        let mut state = WizardState::new(wizard()).unwrap();
        state.set_answer("environment", "production");
        state.advance().unwrap();
        assert_eq!(state.current_index(), 1);
        assert_eq!(state.current_step().name, "review");
    }

    #[test]
    fn advance_on_last_step_errors_out_of_range() {
        let mut state = WizardState::new(wizard()).unwrap();
        state.set_answer("environment", "production");
        state.advance().unwrap();
        assert_eq!(state.advance(), Err(Error::OutOfRange));
    }

    #[test]
    fn back_before_first_step_errors() {
        let mut state = WizardState::new(wizard()).unwrap();
        assert_eq!(state.back(), Err(Error::OutOfRange));
    }

    #[test]
    fn back_then_forward_restores_step() {
        let mut state = WizardState::new(wizard()).unwrap();
        state.set_answer("environment", "production");
        state.advance().unwrap();
        state.back().unwrap();
        assert_eq!(state.current_index(), 0);
    }

    #[test]
    fn is_last_step_true_only_on_final_step() {
        let mut state = WizardState::new(wizard()).unwrap();
        assert!(!state.is_last_step());
        state.set_answer("environment", "production");
        state.advance().unwrap();
        assert!(state.is_last_step());
    }

    #[test]
    fn progress_reflects_completed_and_current_steps() {
        let mut state = WizardState::new(wizard()).unwrap();
        assert_eq!(state.progress(), 0.0);
        state.set_answer("environment", "production");
        assert_eq!(state.progress(), 0.5);
        state.advance().unwrap();
        assert_eq!(state.progress(), 1.0);
    }

    #[test]
    fn blank_answer_counts_as_missing() {
        let mut state = WizardState::new(wizard()).unwrap();
        state.set_answer("environment", "   ");
        assert!(!state.can_advance());
    }
}
