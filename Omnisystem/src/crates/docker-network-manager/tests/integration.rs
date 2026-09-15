use docker_network_manager::{Manager, NetworkMode};

#[test]
fn multi_network_topology_with_reuse() {
    let manager = Manager::new();
    manager.create_network("frontend", NetworkMode::Bridge, Some("172.18.0.0/28")).unwrap();
    manager.create_network("backend", NetworkMode::Overlay, Some("10.0.0.0/28")).unwrap();

    let web_ip = manager.attach("frontend", "web-1").unwrap().unwrap();
    let api_ip_front = manager.attach("frontend", "api-1").unwrap().unwrap();
    let api_ip_back = manager.attach("backend", "api-1").unwrap().unwrap();

    assert_ne!(web_ip, api_ip_front);
    // Same container can hold independent IPs on different networks.
    assert_ne!(api_ip_front, api_ip_back);

    assert_eq!(manager.attached_count("frontend").unwrap(), 2);
    assert_eq!(manager.attached_count("backend").unwrap(), 1);

    manager.detach("frontend", "web-1").unwrap();
    assert_eq!(manager.attached_count("frontend").unwrap(), 1);
    // Freed IP is reusable.
    assert!(manager.attach("frontend", "web-2").is_ok());
}

#[test]
fn host_mode_containers_share_no_subnet() {
    let manager = Manager::new();
    manager.create_network("hostnet", NetworkMode::Host, None).unwrap();
    assert_eq!(manager.attach("hostnet", "c1").unwrap(), None);
    assert_eq!(manager.attach("hostnet", "c2").unwrap(), None);
    assert_eq!(manager.attached_count("hostnet").unwrap(), 2);
}

#[test]
fn exhausted_small_subnet_rejects_further_attachment() {
    let manager = Manager::new();
    manager.create_network("micro", NetworkMode::Bridge, Some("192.168.100.0/30")).unwrap();
    manager.attach("micro", "c1").unwrap();
    manager.attach("micro", "c2").unwrap();
    assert!(manager.attach("micro", "c3").is_err());
}
