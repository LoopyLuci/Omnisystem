//! Network management UI shaping: validating an interface/route snapshot
//! (no route may reference an unknown interface, no duplicate interface
//! names), computing route reachability from live interface state, and
//! summarizing interface throughput for a status panel.

#![warn(missing_docs)]

pub mod error;
pub mod types;

pub use error::{Error, Result};
pub use types::*;

use std::collections::HashMap;

/// Validate that every route references a known interface and that no
/// interface name is duplicated, before the snapshot is displayed.
pub fn validate_snapshot(interfaces: &[Interface], routes: &[Route]) -> Result<()> {
    let mut seen = std::collections::HashSet::new();
    for iface in interfaces {
        if !seen.insert(&iface.name) {
            return Err(Error::DuplicateInterface(iface.name.clone()));
        }
    }
    for route in routes {
        if !interfaces.iter().any(|i| i.name == route.interface) {
            return Err(Error::UnknownInterface(route.interface.clone()));
        }
    }
    Ok(())
}

/// Build display-ready route rows, each carrying whether its interface is
/// currently up. Routes are sorted by metric (ascending), the order a
/// router would prefer them.
pub fn route_table(interfaces: &[Interface], routes: &[Route]) -> Vec<RouteRow> {
    let states: HashMap<&str, LinkState> =
        interfaces.iter().map(|i| (i.name.as_str(), i.state)).collect();
    let mut rows: Vec<RouteRow> = routes
        .iter()
        .map(|r| RouteRow {
            route: r.clone(),
            reachable: states.get(r.interface.as_str()) == Some(&LinkState::Up),
        })
        .collect();
    rows.sort_by_key(|row| row.route.metric);
    rows
}

/// Per-interface throughput summary: total bytes transferred (rx + tx).
pub fn throughput_summary(interfaces: &[Interface]) -> Vec<(String, u64)> {
    let mut out: Vec<(String, u64)> =
        interfaces.iter().map(|i| (i.name.clone(), i.rx_bytes + i.tx_bytes)).collect();
    out.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    out
}

/// Count interfaces that are down, for a health badge.
pub fn down_interface_count(interfaces: &[Interface]) -> usize {
    interfaces.iter().filter(|i| i.state == LinkState::Down).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ifaces() -> Vec<Interface> {
        vec![
            Interface { name: "eth0".into(), state: LinkState::Up, rx_bytes: 1000, tx_bytes: 500 },
            Interface { name: "eth1".into(), state: LinkState::Down, rx_bytes: 0, tx_bytes: 0 },
        ]
    }

    #[test]
    fn validate_passes_for_consistent_snapshot() {
        let routes = vec![Route { destination: "0.0.0.0/0".into(), interface: "eth0".into(), metric: 1 }];
        assert!(validate_snapshot(&ifaces(), &routes).is_ok());
    }

    #[test]
    fn validate_rejects_unknown_interface() {
        let routes = vec![Route { destination: "0.0.0.0/0".into(), interface: "eth9".into(), metric: 1 }];
        assert_eq!(validate_snapshot(&ifaces(), &routes), Err(Error::UnknownInterface("eth9".into())));
    }

    #[test]
    fn validate_rejects_duplicate_interfaces() {
        let dup = vec![ifaces()[0].clone(), ifaces()[0].clone()];
        assert_eq!(validate_snapshot(&dup, &[]), Err(Error::DuplicateInterface("eth0".into())));
    }

    #[test]
    fn route_table_marks_reachability_from_link_state() {
        let routes = vec![
            Route { destination: "10.0.0.0/8".into(), interface: "eth0".into(), metric: 5 },
            Route { destination: "0.0.0.0/0".into(), interface: "eth1".into(), metric: 1 },
        ];
        let rows = route_table(&ifaces(), &routes);
        // metric 1 (eth1) sorts first, but eth1 is down.
        assert!(!rows[0].reachable);
        let eth1_row = rows.iter().find(|r| r.route.interface == "eth1").unwrap();
        assert!(!eth1_row.reachable);
        let eth0_row = rows.iter().find(|r| r.route.interface == "eth0").unwrap();
        assert!(eth0_row.reachable);
    }

    #[test]
    fn route_table_sorted_by_metric_ascending() {
        let routes = vec![
            Route { destination: "a".into(), interface: "eth0".into(), metric: 10 },
            Route { destination: "b".into(), interface: "eth0".into(), metric: 2 },
        ];
        let rows = route_table(&ifaces(), &routes);
        assert_eq!(rows[0].route.destination, "b");
        assert_eq!(rows[1].route.destination, "a");
    }

    #[test]
    fn throughput_summary_sorted_descending_by_total() {
        let summary = throughput_summary(&ifaces());
        assert_eq!(summary[0].0, "eth0");
        assert_eq!(summary[0].1, 1500);
        assert_eq!(summary[1].1, 0);
    }

    #[test]
    fn down_interface_count_counts_correctly() {
        assert_eq!(down_interface_count(&ifaces()), 1);
    }

    #[test]
    fn empty_snapshot_is_valid() {
        assert!(validate_snapshot(&[], &[]).is_ok());
        assert!(route_table(&[], &[]).is_empty());
    }
}
