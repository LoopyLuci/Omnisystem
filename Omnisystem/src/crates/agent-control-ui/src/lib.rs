//! Agent control panel: a registry of controllable agents with a strict
//! lifecycle state machine (`Idle -> Running <-> Paused -> Stopped`) and
//! command validation — rejecting commands that don't apply to the agent's
//! current state, rather than silently applying them.

#![warn(missing_docs)]

pub mod error;
pub mod types;

pub use error::{Error, Result};
pub use types::*;

use std::collections::HashMap;

/// Registry of controlled agents and their lifecycle state.
#[derive(Debug, Default)]
pub struct AgentRegistry {
    agents: HashMap<String, AgentInfo>,
}

impl AgentRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new agent in the `Idle` state.
    pub fn register(&mut self, id: impl Into<String>, name: impl Into<String>) -> Result<()> {
        let id = id.into();
        if self.agents.contains_key(&id) {
            return Err(Error::DuplicateAgent(id));
        }
        self.agents.insert(
            id.clone(),
            AgentInfo { id, name: name.into(), state: AgentState::Idle, commands_applied: 0 },
        );
        Ok(())
    }

    /// Look up an agent's current state.
    pub fn get(&self, id: &str) -> Result<&AgentInfo> {
        self.agents.get(id).ok_or_else(|| Error::UnknownAgent(id.to_string()))
    }

    /// Apply a command to an agent, validating the transition against the
    /// agent's current state. Returns the agent's new state on success.
    pub fn dispatch(&mut self, id: &str, command: Command) -> Result<AgentState> {
        let agent = self.agents.get_mut(id).ok_or_else(|| Error::UnknownAgent(id.to_string()))?;
        let next = match (agent.state, command) {
            (AgentState::Idle, Command::Start) => AgentState::Running,
            (AgentState::Running, Command::Pause) => AgentState::Paused,
            (AgentState::Paused, Command::Resume) => AgentState::Running,
            (AgentState::Running, Command::Stop)
            | (AgentState::Paused, Command::Stop)
            | (AgentState::Idle, Command::Stop) => AgentState::Stopped,
            (from, command) => {
                return Err(Error::InvalidTransition { agent_id: id.to_string(), from, command })
            }
        };
        agent.state = next;
        agent.commands_applied += 1;
        Ok(next)
    }

    /// List every agent currently in the given state.
    pub fn list_by_state(&self, state: AgentState) -> Vec<&AgentInfo> {
        let mut out: Vec<&AgentInfo> = self.agents.values().filter(|a| a.state == state).collect();
        out.sort_by(|a, b| a.id.cmp(&b.id));
        out
    }

    /// Count agents grouped by state, useful for a status-bar summary.
    pub fn state_counts(&self) -> HashMap<AgentState, usize> {
        let mut counts = HashMap::new();
        for agent in self.agents.values() {
            *counts.entry(agent.state).or_insert(0) += 1;
        }
        counts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registers_agent_as_idle() {
        let mut reg = AgentRegistry::new();
        reg.register("a1", "Worker").unwrap();
        assert_eq!(reg.get("a1").unwrap().state, AgentState::Idle);
    }

    #[test]
    fn rejects_duplicate_registration() {
        let mut reg = AgentRegistry::new();
        reg.register("a1", "Worker").unwrap();
        assert_eq!(reg.register("a1", "Worker2"), Err(Error::DuplicateAgent("a1".into())));
    }

    #[test]
    fn start_transitions_idle_to_running() {
        let mut reg = AgentRegistry::new();
        reg.register("a1", "Worker").unwrap();
        assert_eq!(reg.dispatch("a1", Command::Start).unwrap(), AgentState::Running);
    }

    #[test]
    fn full_lifecycle_round_trip() {
        let mut reg = AgentRegistry::new();
        reg.register("a1", "Worker").unwrap();
        reg.dispatch("a1", Command::Start).unwrap();
        reg.dispatch("a1", Command::Pause).unwrap();
        assert_eq!(reg.get("a1").unwrap().state, AgentState::Paused);
        reg.dispatch("a1", Command::Resume).unwrap();
        assert_eq!(reg.get("a1").unwrap().state, AgentState::Running);
        reg.dispatch("a1", Command::Stop).unwrap();
        assert_eq!(reg.get("a1").unwrap().state, AgentState::Stopped);
        assert_eq!(reg.get("a1").unwrap().commands_applied, 4);
    }

    #[test]
    fn rejects_pause_while_idle() {
        let mut reg = AgentRegistry::new();
        reg.register("a1", "Worker").unwrap();
        let err = reg.dispatch("a1", Command::Pause).unwrap_err();
        assert!(matches!(err, Error::InvalidTransition { .. }));
    }

    #[test]
    fn rejects_commands_once_stopped() {
        let mut reg = AgentRegistry::new();
        reg.register("a1", "Worker").unwrap();
        reg.dispatch("a1", Command::Start).unwrap();
        reg.dispatch("a1", Command::Stop).unwrap();
        assert!(reg.dispatch("a1", Command::Start).is_err());
        assert!(reg.dispatch("a1", Command::Resume).is_err());
    }

    #[test]
    fn unknown_agent_errors() {
        let mut reg = AgentRegistry::new();
        assert_eq!(reg.dispatch("nope", Command::Start), Err(Error::UnknownAgent("nope".into())));
    }

    #[test]
    fn list_by_state_filters_and_sorts() {
        let mut reg = AgentRegistry::new();
        reg.register("b", "B").unwrap();
        reg.register("a", "A").unwrap();
        reg.dispatch("a", Command::Start).unwrap();
        let idle = reg.list_by_state(AgentState::Idle);
        assert_eq!(idle.len(), 1);
        assert_eq!(idle[0].id, "b");
    }

    #[test]
    fn state_counts_summarizes_registry() {
        let mut reg = AgentRegistry::new();
        reg.register("a", "A").unwrap();
        reg.register("b", "B").unwrap();
        reg.dispatch("a", Command::Start).unwrap();
        let counts = reg.state_counts();
        assert_eq!(counts.get(&AgentState::Running), Some(&1));
        assert_eq!(counts.get(&AgentState::Idle), Some(&1));
    }
}
