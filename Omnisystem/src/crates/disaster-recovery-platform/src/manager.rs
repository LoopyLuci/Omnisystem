use crate::error::{Error, Result};
use crate::types::{DrillResult, Execution, RecoveryPlan};
use std::collections::HashMap;
use std::sync::RwLock;

/// RPO/RTO-aware disaster-recovery orchestration: register recovery plans,
/// record snapshots taken over time, and drive step-by-step drill (or real
/// recovery) execution, validating both step ordering and whether the
/// observed recovery met the plan's RPO/RTO targets.
pub struct Manager {
    plans: RwLock<HashMap<String, RecoveryPlan>>,
    /// region/system -> sorted snapshot ticks.
    snapshots: RwLock<HashMap<String, Vec<u64>>>,
    executions: RwLock<HashMap<String, Execution>>,
}

impl Manager {
    /// Create a new, empty manager.
    pub fn new() -> Self {
        Self {
            plans: RwLock::new(HashMap::new()),
            snapshots: RwLock::new(HashMap::new()),
            executions: RwLock::new(HashMap::new()),
        }
    }

    /// Register (or replace) a recovery plan.
    pub fn register_plan(&self, plan: RecoveryPlan) {
        self.plans.write().unwrap().insert(plan.name.clone(), plan);
    }

    /// Record that a snapshot of `system` was taken at `tick`.
    pub fn record_snapshot(&self, system: &str, tick: u64) {
        let mut snapshots = self.snapshots.write().unwrap();
        let ticks = snapshots.entry(system.to_string()).or_default();
        ticks.push(tick);
        ticks.sort_unstable();
    }

    /// Most recent snapshot tick at or before `at_tick`, if any.
    fn last_snapshot_before(&self, system: &str, at_tick: u64) -> Option<u64> {
        self.snapshots
            .read()
            .unwrap()
            .get(system)
            .and_then(|ticks| ticks.iter().rev().find(|&&t| t <= at_tick).copied())
    }

    /// Begin executing (drilling or really recovering) `plan_name`, with the
    /// disaster declared at `disaster_tick`.
    pub fn start_drill(&self, plan_name: &str, disaster_tick: u64) -> Result<()> {
        if !self.plans.read().unwrap().contains_key(plan_name) {
            return Err(Error::UnknownPlan(plan_name.to_string()));
        }
        let mut executions = self.executions.write().unwrap();
        if executions.contains_key(plan_name) {
            return Err(Error::DrillAlreadyRunning(plan_name.to_string()));
        }
        executions.insert(
            plan_name.to_string(),
            Execution { started_tick: disaster_tick, next_step: 0 },
        );
        Ok(())
    }

    /// Execute the next step of an in-progress drill. Returns the name of
    /// the step that was just executed.
    pub fn advance_step(&self, plan_name: &str) -> Result<String> {
        let plans = self.plans.read().unwrap();
        let plan = plans.get(plan_name).ok_or_else(|| Error::UnknownPlan(plan_name.to_string()))?;

        let mut executions = self.executions.write().unwrap();
        let exec = executions
            .get_mut(plan_name)
            .ok_or_else(|| Error::NoDrillInProgress(plan_name.to_string()))?;

        if exec.next_step >= plan.steps.len() {
            return Err(Error::StepsNotComplete {
                plan: plan_name.to_string(),
                completed: exec.next_step,
                total: plan.steps.len(),
            });
        }

        let step = plan.steps[exec.next_step].clone();
        exec.next_step += 1;
        Ok(step)
    }

    /// Complete the drill: all steps must already have been executed via
    /// `advance_step`. Computes actual RPO/RTO against the plan's targets,
    /// using the most recent snapshot of `system` at or before the disaster
    /// tick to determine the data-loss window.
    pub fn complete_drill(&self, plan_name: &str, system: &str, completed_tick: u64) -> Result<DrillResult> {
        let plans = self.plans.read().unwrap();
        let plan = plans.get(plan_name).ok_or_else(|| Error::UnknownPlan(plan_name.to_string()))?;

        let mut executions = self.executions.write().unwrap();
        let exec = executions
            .remove(plan_name)
            .ok_or_else(|| Error::NoDrillInProgress(plan_name.to_string()))?;

        if exec.next_step < plan.steps.len() {
            // Put it back -- the caller must finish the steps before completing.
            let completed = exec.next_step;
            let total = plan.steps.len();
            executions.insert(plan_name.to_string(), exec);
            return Err(Error::StepsNotComplete { plan: plan_name.to_string(), completed, total });
        }

        let actual_rto_ticks = completed_tick.saturating_sub(exec.started_tick);
        let rto_met = actual_rto_ticks <= plan.rto_ticks;

        let last_snapshot = self.last_snapshot_before(system, exec.started_tick);
        let actual_rpo_ticks = last_snapshot.map(|s| exec.started_tick.saturating_sub(s));
        let rpo_met = match actual_rpo_ticks {
            Some(gap) => gap <= plan.rpo_ticks,
            None => false,
        };

        Ok(DrillResult {
            plan_name: plan_name.to_string(),
            disaster_tick: exec.started_tick,
            completed_tick,
            actual_rto_ticks,
            rto_met,
            actual_rpo_ticks,
            rpo_met,
            steps_completed: exec.next_step,
            steps_total: plan.steps.len(),
        })
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

    fn plan() -> RecoveryPlan {
        RecoveryPlan {
            name: "db-failover".to_string(),
            rpo_ticks: 10,
            rto_ticks: 30,
            steps: vec![
                "promote replica".to_string(),
                "repoint dns".to_string(),
                "verify writes".to_string(),
            ],
        }
    }

    #[test]
    fn full_drill_within_targets_reports_met() {
        let m = Manager::new();
        m.register_plan(plan());
        m.record_snapshot("db-primary", 95);

        m.start_drill("db-failover", 100).unwrap();
        m.advance_step("db-failover").unwrap();
        m.advance_step("db-failover").unwrap();
        m.advance_step("db-failover").unwrap();
        let result = m.complete_drill("db-failover", "db-primary", 120).unwrap();

        assert_eq!(result.actual_rto_ticks, 20);
        assert!(result.rto_met);
        assert_eq!(result.actual_rpo_ticks, Some(5));
        assert!(result.rpo_met);
        assert_eq!(result.steps_completed, 3);
    }

    #[test]
    fn slow_drill_breaches_rto() {
        let m = Manager::new();
        m.register_plan(plan());
        m.record_snapshot("db-primary", 95);

        m.start_drill("db-failover", 100).unwrap();
        m.advance_step("db-failover").unwrap();
        m.advance_step("db-failover").unwrap();
        m.advance_step("db-failover").unwrap();
        let result = m.complete_drill("db-failover", "db-primary", 200).unwrap();

        assert_eq!(result.actual_rto_ticks, 100);
        assert!(!result.rto_met);
    }

    #[test]
    fn stale_snapshot_breaches_rpo() {
        let m = Manager::new();
        m.register_plan(plan());
        m.record_snapshot("db-primary", 50); // 50 ticks before the disaster, RPO target is 10.

        m.start_drill("db-failover", 100).unwrap();
        m.advance_step("db-failover").unwrap();
        m.advance_step("db-failover").unwrap();
        m.advance_step("db-failover").unwrap();
        let result = m.complete_drill("db-failover", "db-primary", 110).unwrap();

        assert_eq!(result.actual_rpo_ticks, Some(50));
        assert!(!result.rpo_met);
    }

    #[test]
    fn no_snapshot_before_disaster_fails_rpo() {
        let m = Manager::new();
        m.register_plan(plan());
        m.record_snapshot("db-primary", 150); // only a snapshot *after* the disaster.

        m.start_drill("db-failover", 100).unwrap();
        m.advance_step("db-failover").unwrap();
        m.advance_step("db-failover").unwrap();
        m.advance_step("db-failover").unwrap();
        let result = m.complete_drill("db-failover", "db-primary", 110).unwrap();

        assert_eq!(result.actual_rpo_ticks, None);
        assert!(!result.rpo_met);
    }

    #[test]
    fn completing_before_all_steps_run_errors() {
        let m = Manager::new();
        m.register_plan(plan());
        m.start_drill("db-failover", 100).unwrap();
        m.advance_step("db-failover").unwrap();

        let err = m.complete_drill("db-failover", "db-primary", 110).unwrap_err();
        assert!(matches!(err, Error::StepsNotComplete { completed: 1, total: 3, .. }));

        // The execution should still be in progress -- can finish it after.
        m.advance_step("db-failover").unwrap();
        m.advance_step("db-failover").unwrap();
        assert!(m.complete_drill("db-failover", "db-primary", 120).is_ok());
    }

    #[test]
    fn cannot_start_two_drills_concurrently() {
        let m = Manager::new();
        m.register_plan(plan());
        m.start_drill("db-failover", 100).unwrap();
        let err = m.start_drill("db-failover", 105).unwrap_err();
        assert!(matches!(err, Error::DrillAlreadyRunning(_)));
    }

    #[test]
    fn advance_step_on_unknown_plan_errors() {
        let m = Manager::new();
        assert!(matches!(m.advance_step("ghost-plan").unwrap_err(), Error::UnknownPlan(_)));
    }

    #[test]
    fn advance_step_without_start_errors() {
        let m = Manager::new();
        m.register_plan(plan());
        assert!(matches!(m.advance_step("db-failover").unwrap_err(), Error::NoDrillInProgress(_)));
    }

    #[test]
    fn advancing_past_last_step_errors() {
        let m = Manager::new();
        m.register_plan(plan());
        m.start_drill("db-failover", 100).unwrap();
        m.advance_step("db-failover").unwrap();
        m.advance_step("db-failover").unwrap();
        m.advance_step("db-failover").unwrap();
        assert!(matches!(m.advance_step("db-failover").unwrap_err(), Error::StepsNotComplete { .. }));
    }
}
