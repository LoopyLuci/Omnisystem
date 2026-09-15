use crate::error::{Error, Result};
use crate::types::InstanceState;
use std::collections::HashMap;
use std::sync::RwLock;

/// Coordinates zero-downtime instance replacement: an instance only becomes
/// eligible for termination once it has stopped accepting new work
/// (draining) and every in-flight request against it has finished.
pub struct Manager {
    instances: RwLock<HashMap<String, InstanceState>>,
}

impl Manager {
    pub fn new() -> Self {
        Self {
            instances: RwLock::new(HashMap::new()),
        }
    }

    pub fn register_instance(&self, name: &str) {
        self.instances
            .write()
            .unwrap()
            .entry(name.to_string())
            .or_insert_with(|| InstanceState::new(name.to_string()));
    }

    fn with_instance_mut<F, T>(&self, name: &str, f: F) -> Result<T>
    where
        F: FnOnce(&mut InstanceState) -> Result<T>,
    {
        let mut instances = self.instances.write().unwrap();
        let inst = instances
            .get_mut(name)
            .ok_or_else(|| Error::NotFound(name.to_string()))?;
        f(inst)
    }

    pub fn mark_ready(&self, name: &str) -> Result<()> {
        self.with_instance_mut(name, |inst| {
            inst.ready = true;
            Ok(())
        })
    }

    /// Stop routing new requests to this instance. Requires it to have
    /// already been marked ready (i.e. it was actually serving traffic).
    pub fn begin_drain(&self, name: &str) -> Result<()> {
        self.with_instance_mut(name, |inst| {
            if !inst.ready {
                return Err(Error::NotReady(name.to_string()));
            }
            inst.draining = true;
            Ok(())
        })
    }

    pub fn record_request_start(&self, name: &str) -> Result<()> {
        self.with_instance_mut(name, |inst| {
            inst.in_flight_requests += 1;
            Ok(())
        })
    }

    pub fn record_request_end(&self, name: &str) -> Result<()> {
        self.with_instance_mut(name, |inst| {
            inst.in_flight_requests = inst.in_flight_requests.saturating_sub(1);
            Ok(())
        })
    }

    pub fn can_terminate(&self, name: &str) -> Result<bool> {
        let instances = self.instances.read().unwrap();
        let inst = instances
            .get(name)
            .ok_or_else(|| Error::NotFound(name.to_string()))?;
        Ok(inst.draining && inst.in_flight_requests == 0)
    }

    /// Remove the instance once it is fully drained. Errors if it is not
    /// yet safe to terminate.
    pub fn terminate(&self, name: &str) -> Result<()> {
        if !self.can_terminate(name)? {
            return Err(Error::NotDrainable(name.to_string()));
        }
        self.instances.write().unwrap().remove(name);
        Ok(())
    }

    pub fn state(&self, name: &str) -> Result<InstanceState> {
        self.instances
            .read()
            .unwrap()
            .get(name)
            .cloned()
            .ok_or_else(|| Error::NotFound(name.to_string()))
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
    fn drain_requires_ready_first() {
        let m = Manager::new();
        m.register_instance("i1");
        assert!(matches!(m.begin_drain("i1").unwrap_err(), Error::NotReady(_)));
    }

    #[test]
    fn cannot_terminate_while_requests_in_flight() {
        let m = Manager::new();
        m.register_instance("i1");
        m.mark_ready("i1").unwrap();
        m.begin_drain("i1").unwrap();
        m.record_request_start("i1").unwrap();
        assert!(!m.can_terminate("i1").unwrap());
        assert!(matches!(m.terminate("i1").unwrap_err(), Error::NotDrainable(_)));
    }

    #[test]
    fn terminate_succeeds_once_drained() {
        let m = Manager::new();
        m.register_instance("i1");
        m.mark_ready("i1").unwrap();
        m.begin_drain("i1").unwrap();
        m.record_request_start("i1").unwrap();
        m.record_request_start("i1").unwrap();
        m.record_request_end("i1").unwrap();
        assert!(!m.can_terminate("i1").unwrap());
        m.record_request_end("i1").unwrap();
        assert!(m.can_terminate("i1").unwrap());
        m.terminate("i1").unwrap();
        assert!(matches!(m.state("i1").unwrap_err(), Error::NotFound(_)));
    }

    #[test]
    fn cannot_terminate_instance_that_is_not_draining() {
        let m = Manager::new();
        m.register_instance("i1");
        m.mark_ready("i1").unwrap();
        assert!(matches!(m.terminate("i1").unwrap_err(), Error::NotDrainable(_)));
    }

    #[test]
    fn operations_on_unknown_instance_error() {
        let m = Manager::new();
        assert!(matches!(m.mark_ready("ghost").unwrap_err(), Error::NotFound(_)));
    }

    #[test]
    fn in_flight_count_never_underflows() {
        let m = Manager::new();
        m.register_instance("i1");
        m.record_request_end("i1").unwrap();
        assert_eq!(m.state("i1").unwrap().in_flight_requests, 0);
    }
}
