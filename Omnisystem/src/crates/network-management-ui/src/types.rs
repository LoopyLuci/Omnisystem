//! Network resource types.

use serde::{Deserialize, Serialize};

/// Operational status of a network interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LinkState {
    /// Link is up and passing traffic.
    Up,
    /// Link is administratively or physically down.
    Down,
}

/// A network interface as shown in the management UI.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Interface {
    /// Interface name, e.g. `eth0`.
    pub name: String,
    /// Current link state.
    pub state: LinkState,
    /// Bytes received since counters were last reset.
    pub rx_bytes: u64,
    /// Bytes transmitted since counters were last reset.
    pub tx_bytes: u64,
}

/// A routing table entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Route {
    /// Destination CIDR, e.g. `0.0.0.0/0`.
    pub destination: String,
    /// Outgoing interface name.
    pub interface: String,
    /// Route priority; lower wins when destinations overlap.
    pub metric: u32,
}

/// A route enriched with its interface's live state, for a single
/// display-ready row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RouteRow {
    /// The route itself.
    pub route: Route,
    /// Whether the route's interface is currently up.
    pub reachable: bool,
}
