use crate::error::{Error, Result};
use crate::types::{FailoverStatus, RegionState};
use std::collections::HashMap;
use std::sync::RwLock;

/// Multi-region primary/secondary failover, driven by a caller-supplied
/// logical `tick` rather than wall clock time (so behaviour is fully
/// reproducible in tests). One region is designated `original_primary` at
/// construction time. `evaluate` recomputes which regions are alive (have
/// heartbeated within `heartbeat_timeout_ticks` of the tick passed in); if
/// the currently-active primary is no longer alive, traffic fails over to
/// the lexicographically smallest other alive region. If `auto_failback` is
/// enabled and the original primary becomes alive again while a different
/// region is active, traffic fails back to the original primary.
pub struct Manager {
    regions: RwLock<HashMap<String, RegionState>>,
    heartbeat_timeout_ticks: u64,
    original_primary: String,
    active_primary: RwLock<Option<String>>,
    auto_failback: bool,
    failover_count: RwLock<u32>,
}

impl Manager {
    /// Create a manager with `original_primary` as the designated primary region.
    pub fn new(original_primary: &str, heartbeat_timeout_ticks: u64, auto_failback: bool) -> Self {
        Self {
            regions: RwLock::new(HashMap::new()),
            heartbeat_timeout_ticks,
            original_primary: original_primary.to_string(),
            active_primary: RwLock::new(None),
            auto_failback,
            failover_count: RwLock::new(0),
        }
    }

    /// Register a region as part of the failover group.
    pub fn register_region(&self, region_id: &str, tick: u64) {
        self.regions
            .write()
            .unwrap()
            .insert(region_id.to_string(), RegionState { last_heartbeat_tick: tick });
    }

    /// Record a heartbeat from `region_id` at `tick`.
    pub fn heartbeat(&self, region_id: &str, tick: u64) -> Result<()> {
        let mut regions = self.regions.write().unwrap();
        let region = regions
            .get_mut(region_id)
            .ok_or_else(|| Error::UnknownRegion(region_id.to_string()))?;
        region.last_heartbeat_tick = tick;
        Ok(())
    }

    fn alive_regions(&self, tick: u64) -> Vec<String> {
        let regions = self.regions.read().unwrap();
        let mut alive: Vec<String> = regions
            .iter()
            .filter(|(_, state)| tick.saturating_sub(state.last_heartbeat_tick) <= self.heartbeat_timeout_ticks)
            .map(|(id, _)| id.clone())
            .collect();
        alive.sort();
        alive
    }

    /// Recompute the active primary and failover/failback status as of `tick`.
    pub fn evaluate(&self, tick: u64) -> FailoverStatus {
        let total = self.regions.read().unwrap().len();
        let alive = self.alive_regions(tick);

        let mut active = self.active_primary.write().unwrap();
        let mut failover_count = self.failover_count.write().unwrap();

        // Bootstrap: nothing active yet -- start on the original primary if alive.
        if active.is_none() && alive.contains(&self.original_primary) {
            *active = Some(self.original_primary.clone());
        }

        let active_still_alive = active.as_ref().is_some_and(|a| alive.contains(a));

        if !active_still_alive {
            // Fail over to the smallest alive region that isn't the (now-dead) active one.
            let candidate = alive.iter().find(|r| Some(r.as_str()) != active.as_deref()).cloned();
            if candidate.is_some() && candidate != *active {
                *active = candidate;
                *failover_count += 1;
            } else if candidate.is_none() {
                *active = None;
            }
        } else if self.auto_failback
            && active.as_deref() != Some(self.original_primary.as_str())
            && alive.contains(&self.original_primary)
        {
            // Original primary has recovered -- fail back to it.
            *active = Some(self.original_primary.clone());
        }

        FailoverStatus {
            active_primary: active.clone(),
            original_primary: self.original_primary.clone(),
            failed_over: active.as_deref() != Some(self.original_primary.as_str()),
            failover_count: *failover_count,
            alive_regions: alive.len(),
            total_regions: total,
        }
    }

    /// The region currently serving as active primary, if any.
    pub fn current_active_primary(&self) -> Option<String> {
        self.active_primary.read().unwrap().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn group_of_three(auto_failback: bool) -> Manager {
        let m = Manager::new("us-east", 5, auto_failback);
        m.register_region("us-east", 0);
        m.register_region("us-west", 0);
        m.register_region("eu-central", 0);
        m
    }

    #[test]
    fn original_primary_active_when_all_alive() {
        let m = group_of_three(false);
        let status = m.evaluate(1);
        assert_eq!(status.active_primary.as_deref(), Some("us-east"));
        assert!(!status.failed_over);
        assert_eq!(status.alive_regions, 3);
    }

    #[test]
    fn fails_over_when_primary_goes_stale() {
        let m = group_of_three(false);
        m.evaluate(0);
        m.heartbeat("us-west", 10).unwrap();
        m.heartbeat("eu-central", 10).unwrap();
        // us-east never heartbeats again; by tick 12 it's stale.
        let status = m.evaluate(12);
        assert_eq!(status.active_primary.as_deref(), Some("eu-central"));
        assert!(status.failed_over);
        assert_eq!(status.failover_count, 1);
    }

    #[test]
    fn no_failback_when_disabled() {
        let m = group_of_three(false);
        m.evaluate(0);
        m.heartbeat("us-west", 10).unwrap();
        m.heartbeat("eu-central", 10).unwrap();
        m.evaluate(12);
        // us-east recovers.
        m.heartbeat("us-east", 20).unwrap();
        m.heartbeat("us-west", 20).unwrap();
        m.heartbeat("eu-central", 20).unwrap();
        let status = m.evaluate(20);
        assert_eq!(status.active_primary.as_deref(), Some("eu-central"));
        assert!(status.failed_over);
    }

    #[test]
    fn auto_failback_returns_to_original() {
        let m = group_of_three(true);
        m.evaluate(0);
        m.heartbeat("us-west", 10).unwrap();
        m.heartbeat("eu-central", 10).unwrap();
        let after_failover = m.evaluate(12);
        assert_eq!(after_failover.active_primary.as_deref(), Some("eu-central"));

        // us-east recovers.
        m.heartbeat("us-east", 20).unwrap();
        m.heartbeat("us-west", 20).unwrap();
        m.heartbeat("eu-central", 20).unwrap();
        let status = m.evaluate(20);
        assert_eq!(status.active_primary.as_deref(), Some("us-east"));
        assert!(!status.failed_over);
        // Failback doesn't count as an extra failover.
        assert_eq!(status.failover_count, 1);
    }

    #[test]
    fn all_regions_down_leaves_no_active_primary() {
        let m = group_of_three(false);
        m.evaluate(0);
        let status = m.evaluate(100);
        assert_eq!(status.active_primary, None);
        assert_eq!(status.alive_regions, 0);
    }

    #[test]
    fn heartbeat_on_unknown_region_errors() {
        let m = Manager::new("us-east", 5, false);
        assert!(matches!(m.heartbeat("mars", 0).unwrap_err(), Error::UnknownRegion(_)));
    }
}
