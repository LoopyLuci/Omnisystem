//! Container management: a registry enforcing legal lifecycle transitions
//! (`Created -> Running -> Paused/Stopped/Failed`, `Paused -> Running`) and
//! resource-limit health checks.

#![warn(missing_docs)]

use std::collections::HashMap;

pub mod error;
pub mod types;

pub use error::{Error, Result};
pub use types::*;

/// A registry of containers keyed by name.
#[derive(Debug, Clone, Default)]
pub struct Registry {
    containers: HashMap<String, Container>,
}

fn legal_transition(from: ContainerState, to: ContainerState) -> bool {
    use ContainerState::*;
    matches!(
        (from, to),
        (Created, Running)
            | (Running, Paused)
            | (Running, Stopped)
            | (Running, Failed)
            | (Paused, Running)
            | (Paused, Stopped)
            | (Paused, Failed)
    )
}

impl Registry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new container in the `Created` state.
    pub fn register(&mut self, name: &str, resources: ResourceLimits) -> Result<()> {
        if self.containers.contains_key(name) {
            return Err(Error::DuplicateContainer(name.to_string()));
        }
        self.containers
            .insert(name.to_string(), Container { name: name.to_string(), state: ContainerState::Created, resources });
        Ok(())
    }

    /// Transition a container to a new state, rejecting illegal transitions.
    pub fn transition(&mut self, name: &str, to: ContainerState) -> Result<()> {
        let c = self.containers.get_mut(name).ok_or_else(|| Error::UnknownContainer(name.to_string()))?;
        if !legal_transition(c.state, to) {
            return Err(Error::IllegalTransition { name: name.to_string(), from: c.state, to });
        }
        c.state = to;
        Ok(())
    }

    /// List containers currently in a given state, sorted by name.
    pub fn list_by_state(&self, state: ContainerState) -> Vec<&Container> {
        let mut v: Vec<&Container> = self.containers.values().filter(|c| c.state == state).collect();
        v.sort_by(|a, b| a.name.cmp(&b.name));
        v
    }

    /// Look up one container.
    pub fn get(&self, name: &str) -> Result<&Container> {
        self.containers.get(name).ok_or_else(|| Error::UnknownContainer(name.to_string()))
    }
}

/// Whether a container's usage is within both its CPU and memory limits.
pub fn within_limits(c: &Container) -> bool {
    c.resources.cpu_used <= c.resources.cpu_limit && c.resources.memory_used <= c.resources.memory_limit
}

/// Names of running containers that are over either resource limit.
pub fn over_limit_running(registry: &Registry) -> Vec<String> {
    let mut names: Vec<String> = registry
        .list_by_state(ContainerState::Running)
        .into_iter()
        .filter(|c| !within_limits(c))
        .map(|c| c.name.clone())
        .collect();
    names.sort();
    names
}

#[cfg(test)]
mod tests {
    use super::*;

    fn limits(cpu_used: f64, mem_used: f64) -> ResourceLimits {
        ResourceLimits { cpu_limit: 100.0, cpu_used, memory_limit: 512.0, memory_used: mem_used }
    }

    #[test]
    fn register_then_get_returns_created_state() {
        let mut r = Registry::new();
        r.register("web", limits(10.0, 100.0)).unwrap();
        assert_eq!(r.get("web").unwrap().state, ContainerState::Created);
    }

    #[test]
    fn register_rejects_duplicate_name() {
        let mut r = Registry::new();
        r.register("web", limits(0.0, 0.0)).unwrap();
        assert_eq!(r.register("web", limits(0.0, 0.0)), Err(Error::DuplicateContainer("web".into())));
    }

    #[test]
    fn transition_created_to_running_succeeds() {
        let mut r = Registry::new();
        r.register("web", limits(0.0, 0.0)).unwrap();
        r.transition("web", ContainerState::Running).unwrap();
        assert_eq!(r.get("web").unwrap().state, ContainerState::Running);
    }

    #[test]
    fn transition_created_to_stopped_is_illegal() {
        let mut r = Registry::new();
        r.register("web", limits(0.0, 0.0)).unwrap();
        assert!(matches!(r.transition("web", ContainerState::Stopped), Err(Error::IllegalTransition { .. })));
    }

    #[test]
    fn transition_paused_back_to_running_succeeds() {
        let mut r = Registry::new();
        r.register("web", limits(0.0, 0.0)).unwrap();
        r.transition("web", ContainerState::Running).unwrap();
        r.transition("web", ContainerState::Paused).unwrap();
        r.transition("web", ContainerState::Running).unwrap();
        assert_eq!(r.get("web").unwrap().state, ContainerState::Running);
    }

    #[test]
    fn transition_unknown_container_errors() {
        let mut r = Registry::new();
        assert_eq!(r.transition("ghost", ContainerState::Running), Err(Error::UnknownContainer("ghost".into())));
    }

    #[test]
    fn list_by_state_filters_and_sorts() {
        let mut r = Registry::new();
        r.register("b", limits(0.0, 0.0)).unwrap();
        r.register("a", limits(0.0, 0.0)).unwrap();
        r.transition("a", ContainerState::Running).unwrap();
        r.transition("b", ContainerState::Running).unwrap();
        let running = r.list_by_state(ContainerState::Running);
        assert_eq!(running.iter().map(|c| c.name.as_str()).collect::<Vec<_>>(), vec!["a", "b"]);
    }

    #[test]
    fn within_limits_true_when_under_both() {
        let c = Container { name: "web".into(), state: ContainerState::Running, resources: limits(50.0, 200.0) };
        assert!(within_limits(&c));
    }

    #[test]
    fn over_limit_running_detects_cpu_or_memory_breach() {
        let mut r = Registry::new();
        r.register("ok", limits(10.0, 10.0)).unwrap();
        r.register("hot", limits(150.0, 10.0)).unwrap();
        r.transition("ok", ContainerState::Running).unwrap();
        r.transition("hot", ContainerState::Running).unwrap();
        assert_eq!(over_limit_running(&r), vec!["hot".to_string()]);
    }
}
