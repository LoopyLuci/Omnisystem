//! Demo CLI: validate a small network snapshot and print its route table.

use network_management_ui::{route_table, validate_snapshot, Interface, LinkState, Route};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let interfaces = vec![
        Interface { name: "eth0".into(), state: LinkState::Up, rx_bytes: 10_000, tx_bytes: 4_000 },
        Interface { name: "wg0".into(), state: LinkState::Down, rx_bytes: 0, tx_bytes: 0 },
    ];
    let routes = vec![
        Route { destination: "0.0.0.0/0".into(), interface: "eth0".into(), metric: 10 },
        Route { destination: "10.0.0.0/8".into(), interface: "wg0".into(), metric: 1 },
    ];
    validate_snapshot(&interfaces, &routes)?;
    for row in route_table(&interfaces, &routes) {
        println!("{} via {} (reachable: {})", row.route.destination, row.route.interface, row.reachable);
    }
    Ok(())
}
