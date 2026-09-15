use crate::error::{Error, Result};
use crate::types::{CanaryDecision, StageObservation};
use std::sync::RwLock;

/// Staged canary rollout: traffic is shifted to the new version in
/// increasing percentages (`stages`), gated at each step by an error-rate
/// sample supplied by the caller. Exceeding `error_threshold` at any stage
/// immediately reverts traffic to 0%.
pub struct Manager {
    stages: Vec<u8>,
    error_threshold: f64,
    current_stage_idx: RwLock<Option<usize>>,
    history: RwLock<Vec<StageObservation>>,
}

impl Manager {
    /// `stages` must be non-empty and strictly increasing, ending at 100.
    pub fn with_stages(stages: Vec<u8>, error_threshold: f64) -> Self {
        Self {
            stages,
            error_threshold,
            current_stage_idx: RwLock::new(None),
            history: RwLock::new(Vec::new()),
        }
    }

    pub fn new() -> Self {
        Self::with_stages(vec![5, 25, 50, 100], 0.05)
    }

    /// Begin the rollout at the first stage. Returns the traffic percent now
    /// receiving canary traffic.
    pub fn start(&self) -> Result<u8> {
        let mut idx = self.current_stage_idx.write().unwrap();
        if idx.is_some() {
            return Err(Error::AlreadyStarted);
        }
        *idx = Some(0);
        Ok(self.stages[0])
    }

    pub fn current_percent(&self) -> u8 {
        match *self.current_stage_idx.read().unwrap() {
            Some(i) => self.stages[i],
            None => 0,
        }
    }

    /// Feed in an observed error rate for the current stage and decide
    /// whether to advance, complete, or roll back.
    pub fn record_error_rate(&self, error_rate: f64) -> Result<CanaryDecision> {
        let mut idx_guard = self.current_stage_idx.write().unwrap();
        let idx = idx_guard.ok_or(Error::NotStarted)?;
        let stage_percent = self.stages[idx];

        let decision = if error_rate > self.error_threshold {
            *idx_guard = None;
            CanaryDecision::RolledBack
        } else if idx + 1 < self.stages.len() {
            *idx_guard = Some(idx + 1);
            CanaryDecision::Advanced(self.stages[idx + 1])
        } else {
            *idx_guard = None;
            CanaryDecision::Completed
        };

        self.history.write().unwrap().push(StageObservation {
            stage_percent,
            error_rate,
            decision: decision.clone(),
        });
        Ok(decision)
    }

    pub fn history(&self) -> Vec<StageObservation> {
        self.history.read().unwrap().clone()
    }

    pub fn is_in_progress(&self) -> bool {
        self.current_stage_idx.read().unwrap().is_some()
    }
}

impl Default for Manager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_enters_first_stage() {
        let m = Manager::new();
        assert_eq!(m.start().unwrap(), 5);
        assert_eq!(m.current_percent(), 5);
    }

    #[test]
    fn double_start_errors() {
        let m = Manager::new();
        m.start().unwrap();
        assert!(matches!(m.start().unwrap_err(), Error::AlreadyStarted));
    }

    #[test]
    fn recording_before_start_errors() {
        let m = Manager::new();
        assert!(matches!(m.record_error_rate(0.0).unwrap_err(), Error::NotStarted));
    }

    #[test]
    fn low_error_rate_advances_through_all_stages_to_completion() {
        let m = Manager::new();
        m.start().unwrap();
        assert_eq!(m.record_error_rate(0.01).unwrap(), CanaryDecision::Advanced(25));
        assert_eq!(m.record_error_rate(0.01).unwrap(), CanaryDecision::Advanced(50));
        assert_eq!(m.record_error_rate(0.01).unwrap(), CanaryDecision::Advanced(100));
        assert_eq!(m.record_error_rate(0.01).unwrap(), CanaryDecision::Completed);
        assert!(!m.is_in_progress());
    }

    #[test]
    fn high_error_rate_rolls_back_mid_rollout() {
        let m = Manager::new();
        m.start().unwrap();
        m.record_error_rate(0.01).unwrap(); // -> 25%
        let decision = m.record_error_rate(0.2).unwrap();
        assert_eq!(decision, CanaryDecision::RolledBack);
        assert_eq!(m.current_percent(), 0);
        assert!(!m.is_in_progress());
    }

    #[test]
    fn history_records_every_stage_observation() {
        let m = Manager::new();
        m.start().unwrap();
        m.record_error_rate(0.01).unwrap();
        m.record_error_rate(0.2).unwrap();
        assert_eq!(m.history().len(), 2);
    }

    #[test]
    fn can_restart_after_rollback() {
        let m = Manager::new();
        m.start().unwrap();
        m.record_error_rate(0.9).unwrap();
        assert!(!m.is_in_progress());
        assert_eq!(m.start().unwrap(), 5);
    }
}
