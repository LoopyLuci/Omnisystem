use crate::error::{Error, Result};
use crate::types::{CheckConfig, CheckState, HealthStatus};
use std::collections::HashMap;
use std::sync::RwLock;

/// Tracks the health of a set of named checks using consecutive
/// success/failure counters, so a single flaky result doesn't immediately
/// flip status: a check must fail `failure_threshold` times in a row to
/// become `Unhealthy` (one failure alone makes it `Degraded`), and must
/// then succeed `success_threshold` times in a row to become `Healthy`
/// again.
pub struct Manager {
    checks: RwLock<HashMap<String, CheckState>>,
}

impl Manager {
    pub fn new() -> Self {
        Self {
            checks: RwLock::new(HashMap::new()),
        }
    }

    pub fn register(&self, name: &str, config: CheckConfig) {
        self.checks
            .write()
            .unwrap()
            .insert(name.to_string(), CheckState::new(config));
    }

    /// Feed in the outcome of one probe of a check and return its updated
    /// status.
    pub fn record_result(&self, name: &str, success: bool) -> Result<HealthStatus> {
        let mut checks = self.checks.write().unwrap();
        let state = checks
            .get_mut(name)
            .ok_or_else(|| Error::UnknownCheck(name.to_string()))?;

        if success {
            state.consecutive_successes += 1;
            state.consecutive_failures = 0;
            if state.status != HealthStatus::Healthy
                && state.consecutive_successes >= state.config.success_threshold
            {
                state.status = HealthStatus::Healthy;
            }
        } else {
            state.consecutive_failures += 1;
            state.consecutive_successes = 0;
            state.status = if state.consecutive_failures >= state.config.failure_threshold {
                HealthStatus::Unhealthy
            } else {
                HealthStatus::Degraded
            };
        }
        Ok(state.status)
    }

    pub fn status(&self, name: &str) -> Result<HealthStatus> {
        self.checks
            .read()
            .unwrap()
            .get(name)
            .map(|s| s.status)
            .ok_or_else(|| Error::UnknownCheck(name.to_string()))
    }

    /// The worst status across every registered check (Unhealthy beats
    /// Degraded beats Healthy). Healthy if there are no checks registered.
    pub fn aggregate_status(&self) -> HealthStatus {
        self.checks
            .read()
            .unwrap()
            .values()
            .map(|s| s.status)
            .max_by_key(|status| match status {
                HealthStatus::Healthy => 0,
                HealthStatus::Degraded => 1,
                HealthStatus::Unhealthy => 2,
            })
            .unwrap_or(HealthStatus::Healthy)
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
    fn starts_healthy() {
        let m = Manager::new();
        m.register("db", CheckConfig::new(3, 2));
        assert_eq!(m.status("db").unwrap(), HealthStatus::Healthy);
    }

    #[test]
    fn single_failure_degrades_but_does_not_fail() {
        let m = Manager::new();
        m.register("db", CheckConfig::new(3, 2));
        assert_eq!(m.record_result("db", false).unwrap(), HealthStatus::Degraded);
    }

    #[test]
    fn reaching_failure_threshold_marks_unhealthy() {
        let m = Manager::new();
        m.register("db", CheckConfig::new(3, 2));
        m.record_result("db", false).unwrap();
        m.record_result("db", false).unwrap();
        assert_eq!(m.record_result("db", false).unwrap(), HealthStatus::Unhealthy);
    }

    #[test]
    fn a_success_resets_the_failure_streak() {
        let m = Manager::new();
        m.register("db", CheckConfig::new(3, 2));
        m.record_result("db", false).unwrap();
        m.record_result("db", false).unwrap();
        m.record_result("db", true).unwrap();
        // Streak reset, so two more failures are needed to go unhealthy.
        assert_eq!(m.record_result("db", false).unwrap(), HealthStatus::Degraded);
    }

    #[test]
    fn recovery_requires_success_threshold_in_a_row() {
        let m = Manager::new();
        m.register("db", CheckConfig::new(2, 2));
        m.record_result("db", false).unwrap();
        assert_eq!(m.record_result("db", false).unwrap(), HealthStatus::Unhealthy);
        assert_eq!(m.record_result("db", true).unwrap(), HealthStatus::Unhealthy);
        assert_eq!(m.record_result("db", true).unwrap(), HealthStatus::Healthy);
    }

    #[test]
    fn unknown_check_errors() {
        let m = Manager::new();
        assert!(matches!(m.status("ghost").unwrap_err(), Error::UnknownCheck(_)));
    }

    #[test]
    fn aggregate_status_reports_the_worst_check() {
        let m = Manager::new();
        m.register("a", CheckConfig::default());
        m.register("b", CheckConfig::new(2, 2));
        assert_eq!(m.aggregate_status(), HealthStatus::Healthy);
        m.record_result("b", false).unwrap();
        m.record_result("b", false).unwrap();
        assert_eq!(m.aggregate_status(), HealthStatus::Unhealthy);
    }

    #[test]
    fn aggregate_status_with_no_checks_is_healthy() {
        let m = Manager::new();
        assert_eq!(m.aggregate_status(), HealthStatus::Healthy);
    }
}
