use crate::types::{Drift, ResourceId, ResourceKind};
use std::collections::HashMap;
use std::sync::RwLock;

/// Bridges Docker-side observed state (as reported by the individual
/// `docker-*` managers) into Omnisystem's own desired-state model, and
/// reconciles the two: Omnisystem declares what it *wants* a Docker
/// resource's state to be, the Docker-side managers report what they
/// *actually* observe, and `reconcile` finds every discrepancy between the
/// two -- resources Omnisystem wants but were never observed, resources
/// observed that Omnisystem never declared, and resources whose observed
/// state doesn't match what was declared.
pub struct Manager {
    desired: RwLock<HashMap<ResourceId, String>>,
    observed: RwLock<HashMap<ResourceId, String>>,
}

impl Manager {
    /// Create an empty bridge with no desired or observed state yet.
    pub fn new() -> Self {
        Self { desired: RwLock::new(HashMap::new()), observed: RwLock::new(HashMap::new()) }
    }

    /// Declare that Omnisystem wants `id` (of `kind`) to be in `state`.
    pub fn set_desired(&self, kind: ResourceKind, id: &str, state: &str) {
        self.desired.write().unwrap().insert(ResourceId { kind, id: id.to_string() }, state.to_string());
    }

    /// Report that a Docker-side manager observed `id` (of `kind`) to
    /// actually be in `state`.
    pub fn report_observed(&self, kind: ResourceKind, id: &str, state: &str) {
        self.observed.write().unwrap().insert(ResourceId { kind, id: id.to_string() }, state.to_string());
    }

    /// Clear a prior observation (e.g. the resource was removed on the
    /// Docker side).
    pub fn clear_observed(&self, kind: ResourceKind, id: &str) {
        self.observed.write().unwrap().remove(&ResourceId { kind, id: id.to_string() });
    }

    /// Compute every drift between desired and observed state. An empty
    /// result means Docker-side reality matches what Omnisystem wants.
    pub fn reconcile(&self) -> Vec<Drift> {
        let desired = self.desired.read().unwrap();
        let observed = self.observed.read().unwrap();
        let mut drifts = Vec::new();

        for (resource, desired_state) in desired.iter() {
            match observed.get(resource) {
                None => drifts.push(Drift::MissingObservation {
                    resource: resource.clone(),
                    desired_state: desired_state.clone(),
                }),
                Some(observed_state) if observed_state != desired_state => drifts.push(Drift::StateMismatch {
                    resource: resource.clone(),
                    desired_state: desired_state.clone(),
                    observed_state: observed_state.clone(),
                }),
                Some(_) => {}
            }
        }

        for (resource, observed_state) in observed.iter() {
            if !desired.contains_key(resource) {
                drifts.push(Drift::UndeclaredResource { resource: resource.clone(), observed_state: observed_state.clone() });
            }
        }

        drifts
    }

    /// Whether Docker-side reality currently matches everything Omnisystem
    /// has declared as desired (ignoring undeclared/extra resources).
    pub fn is_in_sync(&self) -> bool {
        let desired = self.desired.read().unwrap();
        let observed = self.observed.read().unwrap();
        desired.iter().all(|(resource, state)| observed.get(resource) == Some(state))
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
    fn matching_desired_and_observed_has_no_drift() {
        let m = Manager::new();
        m.set_desired(ResourceKind::Container, "web-1", "running");
        m.report_observed(ResourceKind::Container, "web-1", "running");
        assert!(m.reconcile().is_empty());
        assert!(m.is_in_sync());
    }

    #[test]
    fn missing_observation_is_drift() {
        let m = Manager::new();
        m.set_desired(ResourceKind::Container, "web-1", "running");
        let drifts = m.reconcile();
        assert_eq!(drifts.len(), 1);
        assert!(matches!(&drifts[0], Drift::MissingObservation { .. }));
        assert!(!m.is_in_sync());
    }

    #[test]
    fn state_mismatch_is_drift() {
        let m = Manager::new();
        m.set_desired(ResourceKind::Container, "web-1", "running");
        m.report_observed(ResourceKind::Container, "web-1", "stopped");
        let drifts = m.reconcile();
        assert!(matches!(
            &drifts[0],
            Drift::StateMismatch { desired_state, observed_state, .. }
            if desired_state == "running" && observed_state == "stopped"
        ));
    }

    #[test]
    fn undeclared_resource_is_drift() {
        let m = Manager::new();
        m.report_observed(ResourceKind::Volume, "mystery-vol", "mounted");
        let drifts = m.reconcile();
        assert_eq!(drifts.len(), 1);
        assert!(matches!(&drifts[0], Drift::UndeclaredResource { .. }));
        // Undeclared resources don't count against sync (only desired-vs-observed does).
        assert!(m.is_in_sync());
    }

    #[test]
    fn same_id_different_kind_are_independent_resources() {
        let m = Manager::new();
        m.set_desired(ResourceKind::Container, "app", "running");
        m.set_desired(ResourceKind::Volume, "app", "mounted");
        m.report_observed(ResourceKind::Container, "app", "running");
        // Volume "app" was never observed -- should still drift.
        let drifts = m.reconcile();
        assert_eq!(drifts.len(), 1);
        assert!(matches!(&drifts[0], Drift::MissingObservation { resource, .. } if resource.kind == ResourceKind::Volume));
    }

    #[test]
    fn clearing_an_observation_reintroduces_drift() {
        let m = Manager::new();
        m.set_desired(ResourceKind::Network, "app-net", "active");
        m.report_observed(ResourceKind::Network, "app-net", "active");
        assert!(m.is_in_sync());

        m.clear_observed(ResourceKind::Network, "app-net");
        assert!(!m.is_in_sync());
        assert_eq!(m.reconcile().len(), 1);
    }

    #[test]
    fn multiple_drift_kinds_all_reported_together() {
        let m = Manager::new();
        m.set_desired(ResourceKind::Container, "a", "running");
        m.set_desired(ResourceKind::Container, "b", "running");
        m.report_observed(ResourceKind::Container, "b", "stopped");
        m.report_observed(ResourceKind::Container, "c", "running");

        let drifts = m.reconcile();
        assert_eq!(drifts.len(), 3); // a missing, b mismatch, c undeclared
    }
}
