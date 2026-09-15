use docker_image_manager::{Layer, Manager};

#[test]
fn end_to_end_push_retag_and_resolve() {
    let manager = Manager::new();

    manager
        .push(
            "registry.example.com/team/app:1.0",
            vec![Layer { digest: "sha256:base".to_string(), size_bytes: 10_000 }],
        )
        .unwrap();
    manager
        .push(
            "registry.example.com/team/app:latest",
            vec![
                Layer { digest: "sha256:base".to_string(), size_bytes: 10_000 },
                Layer { digest: "sha256:extra".to_string(), size_bytes: 500 },
            ],
        )
        .unwrap();

    let v1 = manager.resolve_tag("registry.example.com", "team/app", "1.0").unwrap();
    let latest = manager.resolve_tag("registry.example.com", "team/app", "latest").unwrap();
    assert_eq!(v1.total_size_bytes(), 10_000);
    assert_eq!(latest.total_size_bytes(), 10_500);
    assert_eq!(manager.image_count(), 2);

    // Retag 1.0 to point at the same content as latest.
    manager
        .push(
            "registry.example.com/team/app:1.0",
            vec![
                Layer { digest: "sha256:base".to_string(), size_bytes: 10_000 },
                Layer { digest: "sha256:extra".to_string(), size_bytes: 500 },
            ],
        )
        .unwrap();
    let v1_after = manager.resolve_tag("registry.example.com", "team/app", "1.0").unwrap();
    assert_eq!(v1_after.total_size_bytes(), 10_500);
}

#[test]
fn digest_pinned_push_is_resolvable_by_digest() {
    let manager = Manager::new();
    let image = manager.push("myapp@sha256:deadbeef", vec![Layer { digest: "sha256:l1".to_string(), size_bytes: 42 }]).unwrap();
    assert_eq!(image.reference.digest.as_deref(), Some("sha256:deadbeef"));

    let fetched = manager.get_by_digest("sha256:deadbeef").unwrap();
    assert_eq!(fetched.total_size_bytes(), 42);
}

#[test]
fn malformed_reference_is_rejected() {
    let manager = Manager::new();
    assert!(manager.push("registry.example.com/", vec![]).is_err());
}
