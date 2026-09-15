use crate::error::{Error, Result};
use crate::types::{Container, ContainerState};
use std::collections::HashMap;
use std::sync::RwLock;

/// The named action a caller is attempting, used to look up its specific
/// valid-source states -- distinct actions can target the same state (e.g.
/// both `start` and `unpause` end in `Running`) but from disjoint sources,
/// so the check must be keyed on the action, not just the target state.
#[derive(Clone, Copy)]
enum Action {
    Start,
    Pause,
    Unpause,
    Stop,
    Remove,
}

impl Action {
    fn target(self) -> ContainerState {
        use ContainerState::*;
        match self {
            Action::Start => Running,
            Action::Pause => Paused,
            Action::Unpause => Running,
            Action::Stop => Stopped,
            Action::Remove => Removed,
        }
    }

    /// Real-Docker-matching valid source states for this action.
    fn valid_sources(self) -> &'static [ContainerState] {
        use ContainerState::*;
        match self {
            Action::Start => &[Created, Stopped],
            Action::Pause => &[Running],
            Action::Unpause => &[Paused],
            Action::Stop => &[Created, Running, Paused],
            Action::Remove => &[Created, Stopped],
        }
    }
}

/// Tracks containers through Docker's real lifecycle states, rejecting any
/// transition that Docker itself would reject (e.g. pausing a stopped
/// container, or removing a still-running one).
pub struct Manager {
    containers: RwLock<HashMap<String, ContainerState>>,
}

impl Manager {
    /// Create an empty manager.
    pub fn new() -> Self {
        Self { containers: RwLock::new(HashMap::new()) }
    }

    /// Create a new container, tracked in the `Created` state.
    pub fn create(&self, id: &str) -> Result<Container> {
        let mut containers = self.containers.write().unwrap();
        if containers.contains_key(id) {
            return Err(Error::AlreadyExists(id.to_string()));
        }
        containers.insert(id.to_string(), ContainerState::Created);
        Ok(Container { id: id.to_string(), state: ContainerState::Created })
    }

    fn transition(&self, id: &str, action: Action) -> Result<ContainerState> {
        let mut containers = self.containers.write().unwrap();
        let current = containers.get(id).copied().ok_or_else(|| Error::UnknownContainer(id.to_string()))?;

        let to = action.target();
        if !action.valid_sources().contains(&current) {
            return Err(Error::InvalidTransition { id: id.to_string(), from: current, to });
        }
        containers.insert(id.to_string(), to);
        Ok(to)
    }

    /// Start (or restart) a container: `Created`/`Stopped` -> `Running`.
    pub fn start(&self, id: &str) -> Result<ContainerState> {
        self.transition(id, Action::Start)
    }

    /// Pause a running container: `Running` -> `Paused`.
    pub fn pause(&self, id: &str) -> Result<ContainerState> {
        self.transition(id, Action::Pause)
    }

    /// Unpause a paused container: `Paused` -> `Running`.
    pub fn unpause(&self, id: &str) -> Result<ContainerState> {
        self.transition(id, Action::Unpause)
    }

    /// Stop a running, paused, or never-started container: -> `Stopped`.
    pub fn stop(&self, id: &str) -> Result<ContainerState> {
        self.transition(id, Action::Stop)
    }

    /// Remove a stopped or never-started container: -> `Removed`. Removed
    /// containers are dropped from tracking entirely (matches real Docker,
    /// which forgets removed containers).
    pub fn remove(&self, id: &str) -> Result<()> {
        self.transition(id, Action::Remove)?;
        self.containers.write().unwrap().remove(id);
        Ok(())
    }

    /// Look up a container's current state.
    pub fn state_of(&self, id: &str) -> Result<ContainerState> {
        self.containers
            .read()
            .unwrap()
            .get(id)
            .copied()
            .ok_or_else(|| Error::UnknownContainer(id.to_string()))
    }

    /// Count of containers currently in `Running` state.
    pub fn running_count(&self) -> usize {
        self.containers.read().unwrap().values().filter(|s| **s == ContainerState::Running).count()
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
    fn normal_lifecycle_succeeds() {
        let m = Manager::new();
        m.create("c1").unwrap();
        assert_eq!(m.start("c1").unwrap(), ContainerState::Running);
        assert_eq!(m.pause("c1").unwrap(), ContainerState::Paused);
        assert_eq!(m.unpause("c1").unwrap(), ContainerState::Running);
        assert_eq!(m.stop("c1").unwrap(), ContainerState::Stopped);
        assert!(m.remove("c1").is_ok());
        assert!(matches!(m.state_of("c1").unwrap_err(), Error::UnknownContainer(_)));
    }

    #[test]
    fn restart_after_stop_allowed() {
        let m = Manager::new();
        m.create("c1").unwrap();
        m.start("c1").unwrap();
        m.stop("c1").unwrap();
        assert_eq!(m.start("c1").unwrap(), ContainerState::Running);
    }

    #[test]
    fn cannot_pause_a_stopped_container() {
        let m = Manager::new();
        m.create("c1").unwrap();
        m.start("c1").unwrap();
        m.stop("c1").unwrap();
        let err = m.pause("c1").unwrap_err();
        assert!(matches!(err, Error::InvalidTransition { from: ContainerState::Stopped, to: ContainerState::Paused, .. }));
    }

    #[test]
    fn cannot_remove_a_running_container() {
        let m = Manager::new();
        m.create("c1").unwrap();
        m.start("c1").unwrap();
        assert!(matches!(m.remove("c1").unwrap_err(), Error::InvalidTransition { .. }));
    }

    #[test]
    fn duplicate_create_errors() {
        let m = Manager::new();
        m.create("c1").unwrap();
        assert!(matches!(m.create("c1").unwrap_err(), Error::AlreadyExists(_)));
    }

    #[test]
    fn unknown_container_operations_error() {
        let m = Manager::new();
        assert!(matches!(m.start("ghost").unwrap_err(), Error::UnknownContainer(_)));
    }

    #[test]
    fn running_count_tracks_state_changes() {
        let m = Manager::new();
        m.create("c1").unwrap();
        m.create("c2").unwrap();
        m.start("c1").unwrap();
        m.start("c2").unwrap();
        assert_eq!(m.running_count(), 2);
        m.pause("c1").unwrap();
        assert_eq!(m.running_count(), 1);
    }

    #[test]
    fn stop_from_created_without_ever_starting_allowed() {
        let m = Manager::new();
        m.create("c1").unwrap();
        assert_eq!(m.stop("c1").unwrap(), ContainerState::Stopped);
    }
}
