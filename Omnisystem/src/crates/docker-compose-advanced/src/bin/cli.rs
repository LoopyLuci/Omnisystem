//! CLI for docker-compose-advanced.

use docker_compose_advanced::{validate, ComposeFile, Service};
use std::collections::HashMap;

fn main() {
    let mut services = HashMap::new();
    services.insert(
        "web".to_string(),
        Service { image: Some("nginx:latest".to_string()), depends_on: vec!["api".to_string()], ..Default::default() },
    );
    services.insert(
        "api".to_string(),
        Service { image: Some("myorg/api:1.0".to_string()), depends_on: vec!["web".to_string()], ..Default::default() },
    );

    let compose = ComposeFile { services, networks: vec![], volumes: vec![] };
    let issues = validate(&compose);
    if issues.is_empty() {
        println!("compose file is valid");
    } else {
        println!("found {} issue(s):", issues.len());
        for issue in issues {
            println!("  {:?}", issue);
        }
    }
}
