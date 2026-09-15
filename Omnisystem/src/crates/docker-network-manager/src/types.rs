//! Data types
use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;

/// Docker network driver mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkMode {
    /// Local, single-host bridge network (Docker's default).
    Bridge,
    /// Multi-host overlay network (Swarm/Compose multi-node).
    Overlay,
    /// Container shares the host's network namespace directly (no isolation).
    Host,
}

/// A tracked network: its driver mode and IPv4 subnet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Network {
    /// Network name, unique within the manager.
    pub name: String,
    /// Driver mode.
    pub mode: NetworkMode,
    /// CIDR subnet, e.g. `172.18.0.0/24`. `None` for `Host` mode, which has
    /// no subnet of its own.
    pub subnet: Option<Subnet>,
}

/// A parsed IPv4 CIDR subnet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Subnet {
    /// Network base address.
    pub base: Ipv4Addr,
    /// Prefix length, e.g. 24 for a /24.
    pub prefix_len: u8,
}
