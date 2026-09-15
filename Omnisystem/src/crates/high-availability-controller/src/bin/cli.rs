//! CLI demo: form a 3-node cluster, elect a leader, then fail it over.

use high_availability_controller::Manager;

fn main() {
    let m = Manager::new(5);
    m.join("a", 0);
    m.join("b", 0);
    m.join("c", 0);

    let status = m.evaluate(0);
    println!("initial: {status:?}");

    // Node "a" (the leader) stops heartbeating; b and c keep going.
    m.heartbeat("b", 10).unwrap();
    m.heartbeat("c", 10).unwrap();
    let status = m.evaluate(12);
    println!("after failover: {status:?}");
}
