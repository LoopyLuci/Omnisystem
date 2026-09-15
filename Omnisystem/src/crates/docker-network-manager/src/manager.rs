use crate::error::{Error, Result};
use crate::types::{Network, NetworkMode, Subnet};
use std::collections::{HashMap, HashSet};
use std::net::Ipv4Addr;
use std::sync::RwLock;

/// Parse a CIDR string like `172.18.0.0/24` into a [`Subnet`].
pub fn parse_subnet(cidr: &str) -> Result<Subnet> {
    let (addr_str, prefix_str) = cidr.split_once('/').ok_or_else(|| Error::InvalidSubnet(cidr.to_string()))?;
    let base: Ipv4Addr = addr_str.parse().map_err(|_| Error::InvalidSubnet(cidr.to_string()))?;
    let prefix_len: u8 = prefix_str.parse().map_err(|_| Error::InvalidSubnet(cidr.to_string()))?;
    if prefix_len > 32 {
        return Err(Error::InvalidSubnet(cidr.to_string()));
    }
    Ok(Subnet { base, prefix_len })
}

impl Subnet {
    /// Number of usable host addresses (network + broadcast reserved,
    /// matching real subnetting; a /31 or /32 has none usable this way).
    fn usable_host_count(&self) -> u32 {
        if self.prefix_len >= 31 {
            0
        } else {
            (1u32 << (32 - self.prefix_len)) - 2
        }
    }

    /// The address at host offset `n` (1-indexed; offset 0 is the network
    /// address itself and is never allocated).
    fn address_at(&self, offset: u32) -> Ipv4Addr {
        let base_u32 = u32::from(self.base);
        Ipv4Addr::from(base_u32.wrapping_add(offset))
    }
}

struct NetworkState {
    network: Network,
    /// Host offsets (1-indexed within the subnet) already allocated.
    allocated_offsets: HashSet<u32>,
    /// container id -> assigned address.
    attachments: HashMap<String, Ipv4Addr>,
}

/// Tracks Docker networks (bridge/overlay/host mode), allocates IPs from
/// each network's subnet without collision, and tracks which containers are
/// attached to which networks.
pub struct Manager {
    networks: RwLock<HashMap<String, NetworkState>>,
}

impl Manager {
    /// Create an empty manager.
    pub fn new() -> Self {
        Self { networks: RwLock::new(HashMap::new()) }
    }

    /// Create a network. `subnet_cidr` is required for `Bridge`/`Overlay`
    /// modes and ignored (must be `None`) for `Host` mode.
    pub fn create_network(&self, name: &str, mode: NetworkMode, subnet_cidr: Option<&str>) -> Result<()> {
        let mut networks = self.networks.write().unwrap();
        if networks.contains_key(name) {
            return Err(Error::NetworkAlreadyExists(name.to_string()));
        }

        let subnet = match (mode, subnet_cidr) {
            (NetworkMode::Host, _) => None,
            (_, Some(cidr)) => Some(parse_subnet(cidr)?),
            (_, None) => None,
        };

        networks.insert(
            name.to_string(),
            NetworkState {
                network: Network { name: name.to_string(), mode, subnet },
                allocated_offsets: HashSet::new(),
                attachments: HashMap::new(),
            },
        );
        Ok(())
    }

    /// Attach `container` to `network`, allocating it the next free IP from
    /// the network's subnet (host-mode networks attach without an IP).
    pub fn attach(&self, network: &str, container: &str) -> Result<Option<Ipv4Addr>> {
        let mut networks = self.networks.write().unwrap();
        let state = networks.get_mut(network).ok_or_else(|| Error::NetworkNotFound(network.to_string()))?;

        if state.attachments.contains_key(container) {
            return Err(Error::AlreadyAttached { container: container.to_string(), network: network.to_string() });
        }

        let ip = match state.network.mode {
            NetworkMode::Host => None,
            _ => {
                let subnet = state.network.subnet.ok_or_else(|| Error::NoSubnetForHostMode(network.to_string()))?;
                let total = subnet.usable_host_count();
                let offset = (1..=total).find(|o| !state.allocated_offsets.contains(o))
                    .ok_or_else(|| Error::SubnetExhausted(network.to_string()))?;
                state.allocated_offsets.insert(offset);
                Some(subnet.address_at(offset))
            }
        };

        if let Some(addr) = ip {
            state.attachments.insert(container.to_string(), addr);
        } else {
            // Host mode: track attachment with a sentinel (no real address).
            state.attachments.insert(container.to_string(), Ipv4Addr::UNSPECIFIED);
        }

        Ok(ip)
    }

    /// Detach `container` from `network`, freeing its allocated IP if any.
    pub fn detach(&self, network: &str, container: &str) -> Result<()> {
        let mut networks = self.networks.write().unwrap();
        let state = networks.get_mut(network).ok_or_else(|| Error::NetworkNotFound(network.to_string()))?;

        let addr = state
            .attachments
            .remove(container)
            .ok_or_else(|| Error::NotAttached { container: container.to_string(), network: network.to_string() })?;

        if let Some(subnet) = state.network.subnet {
            let base_u32 = u32::from(subnet.base);
            let addr_u32 = u32::from(addr);
            let offset = addr_u32.wrapping_sub(base_u32);
            state.allocated_offsets.remove(&offset);
        }
        Ok(())
    }

    /// Number of containers currently attached to `network`.
    pub fn attached_count(&self, network: &str) -> Result<usize> {
        self.networks
            .read()
            .unwrap()
            .get(network)
            .map(|s| s.attachments.len())
            .ok_or_else(|| Error::NetworkNotFound(network.to_string()))
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
    fn attach_allocates_sequential_ips() {
        let m = Manager::new();
        m.create_network("app-net", NetworkMode::Bridge, Some("172.18.0.0/30")).unwrap();
        // /30 has 2 usable hosts.
        let ip1 = m.attach("app-net", "c1").unwrap().unwrap();
        let ip2 = m.attach("app-net", "c2").unwrap().unwrap();
        assert_ne!(ip1, ip2);
        assert_eq!(m.attached_count("app-net").unwrap(), 2);
    }

    #[test]
    fn subnet_exhaustion_is_detected() {
        let m = Manager::new();
        m.create_network("tiny-net", NetworkMode::Bridge, Some("10.0.0.0/30")).unwrap();
        m.attach("tiny-net", "c1").unwrap();
        m.attach("tiny-net", "c2").unwrap();
        let err = m.attach("tiny-net", "c3").unwrap_err();
        assert!(matches!(err, Error::SubnetExhausted(_)));
    }

    #[test]
    fn detach_frees_the_ip_for_reuse() {
        let m = Manager::new();
        m.create_network("net", NetworkMode::Bridge, Some("10.0.0.0/30")).unwrap();
        m.attach("net", "c1").unwrap();
        m.attach("net", "c2").unwrap();
        m.detach("net", "c1").unwrap();
        // Now there's room for a third container.
        assert!(m.attach("net", "c3").is_ok());
    }

    #[test]
    fn host_mode_network_has_no_subnet_and_no_ip_allocation() {
        let m = Manager::new();
        m.create_network("host-net", NetworkMode::Host, None).unwrap();
        let ip = m.attach("host-net", "c1").unwrap();
        assert_eq!(ip, None);
    }

    #[test]
    fn duplicate_network_name_is_rejected() {
        let m = Manager::new();
        m.create_network("net", NetworkMode::Bridge, Some("10.0.0.0/24")).unwrap();
        assert!(matches!(
            m.create_network("net", NetworkMode::Bridge, Some("10.0.1.0/24")).unwrap_err(),
            Error::NetworkAlreadyExists(_)
        ));
    }

    #[test]
    fn double_attach_same_container_is_rejected() {
        let m = Manager::new();
        m.create_network("net", NetworkMode::Bridge, Some("10.0.0.0/24")).unwrap();
        m.attach("net", "c1").unwrap();
        assert!(matches!(m.attach("net", "c1").unwrap_err(), Error::AlreadyAttached { .. }));
    }

    #[test]
    fn invalid_cidr_is_rejected() {
        let m = Manager::new();
        assert!(m.create_network("net", NetworkMode::Bridge, Some("not-a-cidr")).is_err());
    }

    #[test]
    fn detach_unattached_container_errors() {
        let m = Manager::new();
        m.create_network("net", NetworkMode::Bridge, Some("10.0.0.0/24")).unwrap();
        assert!(matches!(m.detach("net", "ghost").unwrap_err(), Error::NotAttached { .. }));
    }
}
