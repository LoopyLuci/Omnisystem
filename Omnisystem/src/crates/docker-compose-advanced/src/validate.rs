//! Compose file validation: dependency-cycle detection and missing
//! service/network/volume reference checks.

use crate::types::{ComposeFile, ValidationIssue};
use std::collections::{HashMap, HashSet};

/// Validate a compose file, returning every issue found. An empty result
/// means the file is structurally valid.
pub fn validate(compose: &ComposeFile) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    for (name, service) in &compose.services {
        for dep in &service.depends_on {
            if !compose.services.contains_key(dep) {
                issues.push(ValidationIssue::MissingDependency { service: name.clone(), missing: dep.clone() });
            }
        }
        for net in &service.networks {
            if !compose.networks.contains(net) {
                issues.push(ValidationIssue::MissingNetwork { service: name.clone(), missing: net.clone() });
            }
        }
        for vol in &service.volumes {
            // Volume mounts are `volume_name:container_path`; only the part
            // before the first ':' is the named volume to check.
            let vol_name = vol.split(':').next().unwrap_or(vol);
            if !compose.volumes.contains(&vol_name.to_string()) {
                issues.push(ValidationIssue::MissingVolume { service: name.clone(), missing: vol_name.to_string() });
            }
        }
    }

    if let Some(cycle) = find_cycle(compose) {
        issues.push(ValidationIssue::DependencyCycle { services: cycle });
    }

    issues
}

#[derive(Clone, Copy, PartialEq)]
enum VisitState {
    Visiting,
    Done,
}

/// Depth-first search for a dependency cycle among defined services
/// (edges to undefined services are ignored here -- those are reported
/// separately as `MissingDependency`). Returns one cycle, if any exist, as
/// the path of service names from the cycle's start back to itself.
fn find_cycle(compose: &ComposeFile) -> Option<Vec<String>> {
    let mut state: HashMap<&str, VisitState> = HashMap::new();
    let mut stack: Vec<String> = Vec::new();
    let mut stack_set: HashSet<String> = HashSet::new();

    let mut names: Vec<&String> = compose.services.keys().collect();
    names.sort(); // deterministic traversal order for reproducible test output.

    for start in names {
        if state.contains_key(start.as_str()) {
            continue;
        }
        if let Some(cycle) = visit(start, compose, &mut state, &mut stack, &mut stack_set) {
            return Some(cycle);
        }
    }
    None
}

fn visit<'a>(
    node: &'a str,
    compose: &'a ComposeFile,
    state: &mut HashMap<&'a str, VisitState>,
    stack: &mut Vec<String>,
    stack_set: &mut HashSet<String>,
) -> Option<Vec<String>> {
    state.insert(node, VisitState::Visiting);
    stack.push(node.to_string());
    stack_set.insert(node.to_string());

    if let Some(service) = compose.services.get(node) {
        let mut deps: Vec<&String> = service.depends_on.iter().filter(|d| compose.services.contains_key(d.as_str())).collect();
        deps.sort();
        for dep in deps {
            match state.get(dep.as_str()) {
                None => {
                    if let Some(cycle) = visit(dep, compose, state, stack, stack_set) {
                        return Some(cycle);
                    }
                }
                Some(VisitState::Visiting) => {
                    // Found a back-edge: extract the cycle from the stack.
                    let start_idx = stack.iter().position(|s| s == dep).unwrap();
                    let mut cycle: Vec<String> = stack[start_idx..].to_vec();
                    cycle.push(dep.clone());
                    return Some(cycle);
                }
                Some(VisitState::Done) => {}
            }
        }
    }

    stack.pop();
    stack_set.remove(node);
    state.insert(node, VisitState::Done);
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Service;
    use std::collections::HashMap;

    fn service(depends_on: &[&str]) -> Service {
        Service { depends_on: depends_on.iter().map(|s| s.to_string()).collect(), ..Default::default() }
    }

    #[test]
    fn valid_file_has_no_issues() {
        let mut services = HashMap::new();
        services.insert("web".to_string(), service(&["db"]));
        services.insert("db".to_string(), service(&[]));
        let compose = ComposeFile { services, networks: vec![], volumes: vec![] };
        assert!(validate(&compose).is_empty());
    }

    #[test]
    fn missing_dependency_is_reported() {
        let mut services = HashMap::new();
        services.insert("web".to_string(), service(&["db"]));
        let compose = ComposeFile { services, networks: vec![], volumes: vec![] };
        let issues = validate(&compose);
        assert!(issues.contains(&ValidationIssue::MissingDependency { service: "web".to_string(), missing: "db".to_string() }));
    }

    #[test]
    fn simple_two_node_cycle_detected() {
        let mut services = HashMap::new();
        services.insert("a".to_string(), service(&["b"]));
        services.insert("b".to_string(), service(&["a"]));
        let compose = ComposeFile { services, networks: vec![], volumes: vec![] };
        let issues = validate(&compose);
        assert!(issues.iter().any(|i| matches!(i, ValidationIssue::DependencyCycle { .. })));
    }

    #[test]
    fn three_node_cycle_detected() {
        let mut services = HashMap::new();
        services.insert("a".to_string(), service(&["b"]));
        services.insert("b".to_string(), service(&["c"]));
        services.insert("c".to_string(), service(&["a"]));
        let compose = ComposeFile { services, networks: vec![], volumes: vec![] };
        let issues = validate(&compose);
        assert!(issues.iter().any(|i| matches!(i, ValidationIssue::DependencyCycle { .. })));
    }

    #[test]
    fn self_dependency_is_a_cycle() {
        let mut services = HashMap::new();
        services.insert("a".to_string(), service(&["a"]));
        let compose = ComposeFile { services, networks: vec![], volumes: vec![] };
        let issues = validate(&compose);
        assert!(issues.iter().any(|i| matches!(i, ValidationIssue::DependencyCycle { .. })));
    }

    #[test]
    fn missing_network_reference_detected() {
        let mut services = HashMap::new();
        services.insert("web".to_string(), Service { networks: vec!["frontend".to_string()], ..Default::default() });
        let compose = ComposeFile { services, networks: vec![], volumes: vec![] };
        let issues = validate(&compose);
        assert!(issues.contains(&ValidationIssue::MissingNetwork { service: "web".to_string(), missing: "frontend".to_string() }));
    }

    #[test]
    fn missing_volume_reference_detected() {
        let mut services = HashMap::new();
        services.insert("db".to_string(), Service { volumes: vec!["dbdata:/var/lib/db".to_string()], ..Default::default() });
        let compose = ComposeFile { services, networks: vec![], volumes: vec![] };
        let issues = validate(&compose);
        assert!(issues.contains(&ValidationIssue::MissingVolume { service: "db".to_string(), missing: "dbdata".to_string() }));
    }

    #[test]
    fn declared_volume_and_network_pass() {
        let mut services = HashMap::new();
        services.insert(
            "db".to_string(),
            Service { volumes: vec!["dbdata:/var/lib/db".to_string()], networks: vec!["backend".to_string()], ..Default::default() },
        );
        let compose = ComposeFile { services, networks: vec!["backend".to_string()], volumes: vec!["dbdata".to_string()] };
        assert!(validate(&compose).is_empty());
    }

    #[test]
    fn diamond_dependency_graph_is_not_a_false_positive_cycle() {
        let mut services = HashMap::new();
        services.insert("app".to_string(), service(&["db", "cache"]));
        services.insert("db".to_string(), service(&["base"]));
        services.insert("cache".to_string(), service(&["base"]));
        services.insert("base".to_string(), service(&[]));
        let compose = ComposeFile { services, networks: vec![], volumes: vec![] };
        assert!(validate(&compose).is_empty());
    }
}
