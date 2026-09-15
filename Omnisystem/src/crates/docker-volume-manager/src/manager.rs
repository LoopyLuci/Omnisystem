use crate::error::{Error, Result};
use crate::types::{Volume, VolumeState};
use std::collections::HashMap;
use std::sync::RwLock;

struct Mount {
    container: String,
    host_path: String,
}

struct VolumeEntry {
    volume: Volume,
    mounts: Vec<Mount>,
}

/// Tracks Docker volumes through their real lifecycle (create -> mount ->
/// unmount -> remove), detecting host mount-point conflicts (the same host
/// path can't be bind-mounted for two different containers at once) and
/// refusing to remove a volume that's still mounted anywhere.
pub struct Manager {
    volumes: RwLock<HashMap<String, VolumeEntry>>,
    /// host mount path -> (volume, container) currently occupying it.
    mount_points: RwLock<HashMap<String, (String, String)>>,
}

impl Manager {
    /// Create an empty manager.
    pub fn new() -> Self {
        Self { volumes: RwLock::new(HashMap::new()), mount_points: RwLock::new(HashMap::new()) }
    }

    /// Create a new volume in the `Created` state.
    pub fn create(&self, name: &str) -> Result<()> {
        let mut volumes = self.volumes.write().unwrap();
        if volumes.contains_key(name) {
            return Err(Error::AlreadyExists(name.to_string()));
        }
        volumes.insert(
            name.to_string(),
            VolumeEntry { volume: Volume { name: name.to_string(), state: VolumeState::Created }, mounts: Vec::new() },
        );
        Ok(())
    }

    /// Mount `volume` into `container` at `host_path`. Fails if `host_path`
    /// is already occupied by a mount for a different container.
    pub fn mount(&self, volume: &str, container: &str, host_path: &str) -> Result<()> {
        let mut mount_points = self.mount_points.write().unwrap();
        if let Some((_, existing_container)) = mount_points.get(host_path) {
            if existing_container != container {
                return Err(Error::MountPointConflict {
                    path: host_path.to_string(),
                    existing_container: existing_container.clone(),
                });
            }
        }

        let mut volumes = self.volumes.write().unwrap();
        let entry = volumes.get_mut(volume).ok_or_else(|| Error::NotFound(volume.to_string()))?;
        entry.mounts.push(Mount { container: container.to_string(), host_path: host_path.to_string() });
        entry.volume.state = VolumeState::Mounted;

        mount_points.insert(host_path.to_string(), (volume.to_string(), container.to_string()));
        Ok(())
    }

    /// Unmount `volume` from `container`. If no other container has it
    /// mounted, the volume transitions to `Unmounted`.
    pub fn unmount(&self, volume: &str, container: &str) -> Result<()> {
        let mut volumes = self.volumes.write().unwrap();
        let entry = volumes.get_mut(volume).ok_or_else(|| Error::NotFound(volume.to_string()))?;

        let before = entry.mounts.len();
        let mut mount_points = self.mount_points.write().unwrap();
        entry.mounts.retain(|m| {
            if m.container == container {
                mount_points.remove(&m.host_path);
                false
            } else {
                true
            }
        });
        if entry.mounts.len() == before {
            return Err(Error::NotFound(format!("{volume}:{container}")));
        }

        if entry.mounts.is_empty() {
            entry.volume.state = VolumeState::Unmounted;
        }
        Ok(())
    }

    /// Remove `volume`. Fails if it's still mounted anywhere.
    pub fn remove(&self, volume: &str) -> Result<()> {
        let mut volumes = self.volumes.write().unwrap();
        let entry = volumes.get(volume).ok_or_else(|| Error::NotFound(volume.to_string()))?;
        if !entry.mounts.is_empty() {
            return Err(Error::VolumeInUse(volume.to_string()));
        }
        volumes.remove(volume);
        Ok(())
    }

    /// Current lifecycle state of `volume`.
    pub fn state_of(&self, volume: &str) -> Result<VolumeState> {
        self.volumes.read().unwrap().get(volume).map(|e| e.volume.state).ok_or_else(|| Error::NotFound(volume.to_string()))
    }

    /// Number of active mounts for `volume`.
    pub fn mount_count(&self, volume: &str) -> Result<usize> {
        self.volumes.read().unwrap().get(volume).map(|e| e.mounts.len()).ok_or_else(|| Error::NotFound(volume.to_string()))
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
    fn full_lifecycle_succeeds() {
        let m = Manager::new();
        m.create("data").unwrap();
        assert_eq!(m.state_of("data").unwrap(), VolumeState::Created);

        m.mount("data", "c1", "/var/lib/data").unwrap();
        assert_eq!(m.state_of("data").unwrap(), VolumeState::Mounted);

        m.unmount("data", "c1").unwrap();
        assert_eq!(m.state_of("data").unwrap(), VolumeState::Unmounted);

        m.remove("data").unwrap();
        assert!(m.state_of("data").is_err());
    }

    #[test]
    fn cannot_remove_a_mounted_volume() {
        let m = Manager::new();
        m.create("data").unwrap();
        m.mount("data", "c1", "/var/lib/data").unwrap();
        assert!(matches!(m.remove("data").unwrap_err(), Error::VolumeInUse(_)));
    }

    #[test]
    fn conflicting_mount_point_rejected() {
        let m = Manager::new();
        m.create("data1").unwrap();
        m.create("data2").unwrap();
        m.mount("data1", "c1", "/shared").unwrap();
        let err = m.mount("data2", "c2", "/shared").unwrap_err();
        assert!(matches!(err, Error::MountPointConflict { .. }));
    }

    #[test]
    fn same_container_reusing_its_own_mount_point_is_fine() {
        let m = Manager::new();
        m.create("data").unwrap();
        m.mount("data", "c1", "/mnt").unwrap();
        // Same container, same path, different volume name -- shouldn't conflict
        // with itself (e.g. remounting after a restart).
        assert!(m.mount("data", "c1", "/mnt").is_ok());
    }

    #[test]
    fn stays_mounted_state_while_any_container_holds_it() {
        let m = Manager::new();
        m.create("data").unwrap();
        m.mount("data", "c1", "/a").unwrap();
        m.mount("data", "c2", "/b").unwrap();
        m.unmount("data", "c1").unwrap();
        assert_eq!(m.state_of("data").unwrap(), VolumeState::Mounted);
        assert_eq!(m.mount_count("data").unwrap(), 1);
    }

    #[test]
    fn duplicate_create_rejected() {
        let m = Manager::new();
        m.create("data").unwrap();
        assert!(matches!(m.create("data").unwrap_err(), Error::AlreadyExists(_)));
    }

    #[test]
    fn unmount_of_nonexistent_mount_errors() {
        let m = Manager::new();
        m.create("data").unwrap();
        assert!(m.unmount("data", "ghost").is_err());
    }

    #[test]
    fn freeing_a_mount_point_allows_reuse_by_another_volume() {
        let m = Manager::new();
        m.create("data1").unwrap();
        m.create("data2").unwrap();
        m.mount("data1", "c1", "/shared").unwrap();
        m.unmount("data1", "c1").unwrap();
        assert!(m.mount("data2", "c2", "/shared").is_ok());
    }
}
