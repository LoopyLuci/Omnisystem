//! Data types
use serde::{Deserialize, Serialize};

/// The kind of Docker resource being bridged into Omnisystem's state model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceKind {
    /// A container tracked by `docker-container-lifecycle`.
    Container,
    /// An image tracked by `docker-image-manager`.
    Image,
    /// A network tracked by `docker-network-manager`.
    Network,
    /// A volume tracked by `docker-volume-manager`.
    Volume,
}

/// Identity of a single Docker resource within a kind.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResourceId {
    /// Which kind of resource this is.
    pub kind: ResourceKind,
    /// The resource's id/name within its kind.
    pub id: String,
}

/// A discrepancy found between Omnisystem's desired state for a Docker
/// resource and what was actually observed from the Docker-side managers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Drift {
    /// Omnisystem expects this resource to exist in this state, but no
    /// observation was ever reported for it.
    MissingObservation {
        /// The under-observed resource.
        resource: ResourceId,
        /// The state Omnisystem expects it to be in.
        desired_state: String,
    },
    /// A resource was observed that Omnisystem never declared as desired
    /// (drift in the other direction -- something exists that shouldn't,
    /// from Omnisystem's point of view).
    UndeclaredResource {
        /// The unexpected resource.
        resource: ResourceId,
        /// The state it was observed in.
        observed_state: String,
    },
    /// The resource exists on both sides but its state doesn't match.
    StateMismatch {
        /// The resource whose state disagrees.
        resource: ResourceId,
        /// The state Omnisystem expects it to be in.
        desired_state: String,
        /// The state actually observed.
        observed_state: String,
    },
}
