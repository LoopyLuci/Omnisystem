use docker_volume_manager::{Manager, VolumeState};

#[test]
fn end_to_end_shared_volume_across_containers() {
    let manager = Manager::new();
    manager.create("shared-cache").unwrap();

    manager.mount("shared-cache", "worker-1", "/cache").unwrap();
    manager.mount("shared-cache", "worker-2", "/cache-worker2").unwrap();
    assert_eq!(manager.mount_count("shared-cache").unwrap(), 2);
    assert_eq!(manager.state_of("shared-cache").unwrap(), VolumeState::Mounted);

    manager.unmount("shared-cache", "worker-1").unwrap();
    assert_eq!(manager.state_of("shared-cache").unwrap(), VolumeState::Mounted);

    manager.unmount("shared-cache", "worker-2").unwrap();
    assert_eq!(manager.state_of("shared-cache").unwrap(), VolumeState::Unmounted);

    manager.remove("shared-cache").unwrap();
    assert!(manager.state_of("shared-cache").is_err());
}

#[test]
fn mount_point_conflict_across_volumes_is_caught() {
    let manager = Manager::new();
    manager.create("vol-a").unwrap();
    manager.create("vol-b").unwrap();
    manager.mount("vol-a", "container-1", "/data").unwrap();

    let err = manager.mount("vol-b", "container-2", "/data").unwrap_err();
    assert!(err.to_string().contains("already in use"));
}

#[test]
fn cannot_remove_while_mounted() {
    let manager = Manager::new();
    manager.create("vol").unwrap();
    manager.mount("vol", "c1", "/x").unwrap();
    assert!(manager.remove("vol").is_err());
}
