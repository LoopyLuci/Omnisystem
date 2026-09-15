//! Automation builder: validates a [`Workflow`]'s step dependency graph and
//! produces a deterministic topological execution order (Kahn's algorithm,
//! ties broken by step name so results are reproducible).

#![warn(missing_docs)]

use std::collections::{BTreeMap, BTreeSet, HashMap};

pub mod error;
pub mod types;

pub use error::{Error, Result};
pub use types::*;

/// Validate that a workflow has no duplicate step names and every
/// dependency refers to a step that exists.
pub fn validate(workflow: &Workflow) -> Result<()> {
    let mut seen = BTreeSet::new();
    for step in &workflow.steps {
        if !seen.insert(step.name.clone()) {
            return Err(Error::DuplicateStep(step.name.clone()));
        }
    }
    for step in &workflow.steps {
        for dep in &step.depends_on {
            if !seen.contains(dep) {
                return Err(Error::UnknownDependency { step: step.name.clone(), depends_on: dep.clone() });
            }
        }
    }
    Ok(())
}

/// Compute a valid execution order: each step appears only after every
/// step it depends on. Errors on duplicate names, unknown dependencies, or
/// a cycle.
pub fn topological_order(workflow: &Workflow) -> Result<Vec<String>> {
    validate(workflow)?;

    let mut in_degree: HashMap<&str, usize> = HashMap::new();
    let mut dependents: HashMap<&str, Vec<&str>> = HashMap::new();
    for step in &workflow.steps {
        in_degree.entry(&step.name).or_insert(0);
        for dep in &step.depends_on {
            *in_degree.entry(&step.name).or_insert(0) += 1;
            dependents.entry(dep.as_str()).or_default().push(&step.name);
        }
    }

    // Use a BTreeMap-backed ready set so the pop order is deterministic
    // (lexicographic among currently-ready steps).
    let mut ready: BTreeSet<&str> =
        in_degree.iter().filter(|(_, &d)| d == 0).map(|(&k, _)| k).collect();
    let mut remaining = in_degree.clone();
    let mut order = Vec::with_capacity(workflow.steps.len());

    while let Some(&name) = ready.iter().next() {
        ready.remove(name);
        order.push(name.to_string());
        if let Some(deps) = dependents.get(name) {
            let mut deps: Vec<&str> = deps.clone();
            deps.sort();
            for d in deps {
                let entry = remaining.get_mut(d).unwrap();
                *entry -= 1;
                if *entry == 0 {
                    ready.insert(d);
                }
            }
        }
    }

    if order.len() != workflow.steps.len() {
        let unresolved: Vec<String> =
            workflow.steps.iter().map(|s| s.name.clone()).filter(|n| !order.contains(n)).collect();
        return Err(Error::CycleDetected(unresolved));
    }
    Ok(order)
}

/// Group steps into "waves" that can run in parallel: wave 0 has no
/// dependencies, wave N depends only on steps in waves `< N`.
pub fn parallel_waves(workflow: &Workflow) -> Result<Vec<Vec<String>>> {
    let order = topological_order(workflow)?;
    let by_name: BTreeMap<&str, &Step> = workflow.steps.iter().map(|s| (s.name.as_str(), s)).collect();
    let mut wave_of: HashMap<String, usize> = HashMap::new();
    let mut waves: Vec<Vec<String>> = Vec::new();

    for name in order {
        let step = by_name[name.as_str()];
        let wave = step.depends_on.iter().map(|d| wave_of[d] + 1).max().unwrap_or(0);
        wave_of.insert(name.clone(), wave);
        if waves.len() <= wave {
            waves.resize_with(wave + 1, Vec::new);
        }
        waves[wave].push(name);
    }
    for wave in &mut waves {
        wave.sort();
    }
    Ok(waves)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn step(name: &str, deps: &[&str]) -> Step {
        Step { name: name.into(), depends_on: deps.iter().map(|s| s.to_string()).collect() }
    }

    #[test]
    fn validate_rejects_duplicate_step_names() {
        let wf = Workflow { steps: vec![step("a", &[]), step("a", &[])] };
        assert_eq!(validate(&wf), Err(Error::DuplicateStep("a".into())));
    }

    #[test]
    fn validate_rejects_unknown_dependency() {
        let wf = Workflow { steps: vec![step("a", &["ghost"])] };
        assert!(matches!(validate(&wf), Err(Error::UnknownDependency { .. })));
    }

    #[test]
    fn topological_order_respects_dependencies() {
        let wf = Workflow { steps: vec![step("build", &[]), step("test", &["build"]), step("deploy", &["test"])] };
        let order = topological_order(&wf).unwrap();
        assert_eq!(order, vec!["build", "test", "deploy"]);
    }

    #[test]
    fn topological_order_is_deterministic_for_ties() {
        let wf = Workflow { steps: vec![step("b", &[]), step("a", &[]), step("c", &[])] };
        let order = topological_order(&wf).unwrap();
        assert_eq!(order, vec!["a", "b", "c"]);
    }

    #[test]
    fn topological_order_detects_cycle() {
        let wf = Workflow { steps: vec![step("a", &["b"]), step("b", &["a"])] };
        assert!(matches!(topological_order(&wf), Err(Error::CycleDetected(_))));
    }

    #[test]
    fn topological_order_detects_self_cycle() {
        let wf = Workflow { steps: vec![step("a", &["a"])] };
        assert!(matches!(topological_order(&wf), Err(Error::CycleDetected(_))));
    }

    #[test]
    fn parallel_waves_groups_independent_steps() {
        let wf = Workflow {
            steps: vec![step("lint", &[]), step("unit-test", &[]), step("package", &["lint", "unit-test"])],
        };
        let waves = parallel_waves(&wf).unwrap();
        assert_eq!(waves.len(), 2);
        assert_eq!(waves[0], vec!["lint", "unit-test"]);
        assert_eq!(waves[1], vec!["package"]);
    }

    #[test]
    fn parallel_waves_single_chain_is_one_step_per_wave() {
        let wf = Workflow { steps: vec![step("a", &[]), step("b", &["a"]), step("c", &["b"])] };
        let waves = parallel_waves(&wf).unwrap();
        assert_eq!(waves, vec![vec!["a".to_string()], vec!["b".to_string()], vec!["c".to_string()]]);
    }

    #[test]
    fn empty_workflow_has_empty_order() {
        let wf = Workflow::default();
        assert!(topological_order(&wf).unwrap().is_empty());
    }
}
