//! Data types
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single service definition within a compose file.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Service {
    /// Image to run (mutually informative with `build`, not validated here).
    pub image: Option<String>,
    /// Names of networks this service is connected to (must exist in the
    /// top-level `networks` map).
    pub networks: Vec<String>,
    /// Volume mounts, as `volume_name:container_path` (the volume name must
    /// exist in the top-level `volumes` map).
    pub volumes: Vec<String>,
    /// Names of other services this service depends on.
    pub depends_on: Vec<String>,
}

/// A full compose file model: named services, networks, and volumes.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComposeFile {
    /// Service name -> definition.
    pub services: HashMap<String, Service>,
    /// Declared network names.
    pub networks: Vec<String>,
    /// Declared volume names.
    pub volumes: Vec<String>,
}

/// A single validation problem found in a compose file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationIssue {
    /// `service` depends on a service name that isn't defined.
    MissingDependency {
        /// The service with the dangling dependency.
        service: String,
        /// The dependency name that isn't defined.
        missing: String,
    },
    /// `service` references a network that isn't declared.
    MissingNetwork {
        /// The service referencing the network.
        service: String,
        /// The network name that isn't declared.
        missing: String,
    },
    /// `service` references a volume that isn't declared.
    MissingVolume {
        /// The service referencing the volume.
        service: String,
        /// The volume name that isn't declared.
        missing: String,
    },
    /// A dependency cycle was found among these services (in cycle order).
    DependencyCycle {
        /// The services forming the cycle, in order.
        services: Vec<String>,
    },
}
