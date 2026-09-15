use docker_compose_advanced::{validate, ComposeFile, Service, ValidationIssue};
use std::collections::HashMap;

#[test]
fn realistic_web_app_compose_file_is_valid() {
    let mut services = HashMap::new();
    services.insert(
        "web".to_string(),
        Service {
            image: Some("nginx:latest".to_string()),
            networks: vec!["frontend".to_string()],
            depends_on: vec!["api".to_string()],
            volumes: vec![],
        },
    );
    services.insert(
        "api".to_string(),
        Service {
            image: Some("myorg/api:1.0".to_string()),
            networks: vec!["frontend".to_string(), "backend".to_string()],
            depends_on: vec!["db".to_string()],
            volumes: vec![],
        },
    );
    services.insert(
        "db".to_string(),
        Service {
            image: Some("postgres:16".to_string()),
            networks: vec!["backend".to_string()],
            depends_on: vec![],
            volumes: vec!["dbdata:/var/lib/postgresql/data".to_string()],
        },
    );

    let compose = ComposeFile {
        services,
        networks: vec!["frontend".to_string(), "backend".to_string()],
        volumes: vec!["dbdata".to_string()],
    };

    assert!(validate(&compose).is_empty());
}

#[test]
fn compose_file_with_multiple_kinds_of_issues_reports_all() {
    let mut services = HashMap::new();
    services.insert(
        "web".to_string(),
        Service {
            image: Some("nginx".to_string()),
            networks: vec!["nonexistent-net".to_string()],
            depends_on: vec!["ghost-service".to_string()],
            volumes: vec!["nonexistent-vol:/data".to_string()],
        },
    );

    let compose = ComposeFile { services, networks: vec![], volumes: vec![] };
    let issues = validate(&compose);

    assert!(issues.contains(&ValidationIssue::MissingDependency {
        service: "web".to_string(),
        missing: "ghost-service".to_string()
    }));
    assert!(issues.contains(&ValidationIssue::MissingNetwork {
        service: "web".to_string(),
        missing: "nonexistent-net".to_string()
    }));
    assert!(issues.contains(&ValidationIssue::MissingVolume {
        service: "web".to_string(),
        missing: "nonexistent-vol".to_string()
    }));
    assert_eq!(issues.len(), 3);
}

#[test]
fn circular_dependency_across_three_services_is_caught() {
    let mut services = HashMap::new();
    services.insert("frontend".to_string(), Service { depends_on: vec!["backend".to_string()], ..Default::default() });
    services.insert("backend".to_string(), Service { depends_on: vec!["cache".to_string()], ..Default::default() });
    services.insert("cache".to_string(), Service { depends_on: vec!["frontend".to_string()], ..Default::default() });

    let compose = ComposeFile { services, networks: vec![], volumes: vec![] };
    let issues = validate(&compose);
    assert!(issues.iter().any(|i| matches!(i, ValidationIssue::DependencyCycle { .. })));
}
