use crate::error::{Error, Result};
use crate::types::{Environment, EnvironmentState, SwitchEvent};
use chrono::Utc;
use std::sync::RwLock;

/// Orchestrates a blue/green deployment: one slot is "active" (serving
/// traffic) and the other is "standby". A new version is deployed to the
/// standby slot, health-checked, and then promoted to active by an atomic
/// pointer flip. Every promotion is recorded so the previous active
/// environment can be restored with `rollback`.
pub struct Manager {
    active: RwLock<Environment>,
    blue: RwLock<EnvironmentState>,
    green: RwLock<EnvironmentState>,
    history: RwLock<Vec<SwitchEvent>>,
}

impl Manager {
    pub fn new() -> Self {
        Self {
            active: RwLock::new(Environment::Blue),
            blue: RwLock::new(EnvironmentState::new(Environment::Blue)),
            green: RwLock::new(EnvironmentState::new(Environment::Green)),
            history: RwLock::new(Vec::new()),
        }
    }

    fn state_for(&self, env: Environment) -> &RwLock<EnvironmentState> {
        match env {
            Environment::Blue => &self.blue,
            Environment::Green => &self.green,
        }
    }

    pub fn active_environment(&self) -> Environment {
        *self.active.read().unwrap()
    }

    pub fn active_state(&self) -> EnvironmentState {
        self.state_for(self.active_environment()).read().unwrap().clone()
    }

    pub fn standby_state(&self) -> EnvironmentState {
        self.state_for(self.active_environment().other())
            .read()
            .unwrap()
            .clone()
    }

    /// Deploy a new version onto the standby slot. It starts out unhealthy
    /// until `mark_standby_healthy` (or an external health check) says
    /// otherwise.
    pub fn deploy_to_standby(&self, version: String) -> Result<EnvironmentState> {
        let standby = self.active_environment().other();
        let mut guard = self.state_for(standby).write().unwrap();
        guard.version = version;
        guard.healthy = false;
        Ok(guard.clone())
    }

    /// Record that the standby slot has passed its health checks.
    pub fn mark_standby_healthy(&self) -> Result<()> {
        let standby = self.active_environment().other();
        self.state_for(standby).write().unwrap().healthy = true;
        Ok(())
    }

    /// Flip traffic to the standby slot. Requires the standby to be healthy.
    pub fn promote(&self) -> Result<SwitchEvent> {
        let standby = self.active_environment().other();
        let standby_state = self.state_for(standby).read().unwrap().clone();
        if !standby_state.healthy {
            return Err(Error::StandbyNotHealthy);
        }
        let previous_active = self.active_environment();
        *self.active.write().unwrap() = standby;
        let event = SwitchEvent {
            from: previous_active,
            to: standby,
            version: standby_state.version,
            at: Utc::now(),
            is_rollback: false,
        };
        self.history.write().unwrap().push(event.clone());
        Ok(event)
    }

    /// Undo the most recent promotion by flipping traffic back to whichever
    /// environment was active before it.
    pub fn rollback(&self) -> Result<SwitchEvent> {
        let mut history = self.history.write().unwrap();
        let last = history
            .iter()
            .rev()
            .find(|e| !e.is_rollback)
            .cloned()
            .ok_or(Error::NoRollbackTarget)?;
        if self.active_environment() != last.to {
            return Err(Error::NoRollbackTarget);
        }
        let restored_version = self.state_for(last.from).read().unwrap().version.clone();
        *self.active.write().unwrap() = last.from;
        let event = SwitchEvent {
            from: last.to,
            to: last.from,
            version: restored_version,
            at: Utc::now(),
            is_rollback: true,
        };
        history.push(event.clone());
        Ok(event)
    }

    pub fn history(&self) -> Vec<SwitchEvent> {
        self.history.read().unwrap().clone()
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
    fn starts_on_blue_with_no_deployed_version() {
        let m = Manager::new();
        assert_eq!(m.active_environment(), Environment::Blue);
        assert_eq!(m.active_state().version, "unset");
    }

    #[test]
    fn promote_fails_when_standby_unhealthy() {
        let m = Manager::new();
        m.deploy_to_standby("1.1.0".to_string()).unwrap();
        let err = m.promote().unwrap_err();
        assert!(matches!(err, Error::StandbyNotHealthy));
        assert_eq!(m.active_environment(), Environment::Blue);
    }

    #[test]
    fn promote_switches_active_after_health_check() {
        let m = Manager::new();
        m.deploy_to_standby("1.1.0".to_string()).unwrap();
        m.mark_standby_healthy().unwrap();
        let event = m.promote().unwrap();
        assert_eq!(event.from, Environment::Blue);
        assert_eq!(event.to, Environment::Green);
        assert_eq!(m.active_environment(), Environment::Green);
        assert_eq!(m.active_state().version, "1.1.0");
    }

    #[test]
    fn rollback_restores_previous_active_environment() {
        let m = Manager::new();
        m.deploy_to_standby("1.1.0".to_string()).unwrap();
        m.mark_standby_healthy().unwrap();
        m.promote().unwrap();
        assert_eq!(m.active_environment(), Environment::Green);

        let event = m.rollback().unwrap();
        assert!(event.is_rollback);
        assert_eq!(m.active_environment(), Environment::Blue);
    }

    #[test]
    fn rollback_without_prior_promotion_errors() {
        let m = Manager::new();
        assert!(matches!(m.rollback().unwrap_err(), Error::NoRollbackTarget));
    }

    #[test]
    fn double_rollback_errors_second_time() {
        let m = Manager::new();
        m.deploy_to_standby("1.1.0".to_string()).unwrap();
        m.mark_standby_healthy().unwrap();
        m.promote().unwrap();
        m.rollback().unwrap();
        assert!(matches!(m.rollback().unwrap_err(), Error::NoRollbackTarget));
    }

    #[test]
    fn history_accumulates_events() {
        let m = Manager::new();
        m.deploy_to_standby("1.1.0".to_string()).unwrap();
        m.mark_standby_healthy().unwrap();
        m.promote().unwrap();
        assert_eq!(m.history().len(), 1);
    }
}
