//! CLI demo: register a check, feed it some failures, then recover it.

use health_check_engine::{CheckConfig, Manager};

fn main() {
    let m = Manager::new();
    m.register("db", CheckConfig::new(3, 2));

    for ok in [true, false, false, false, true, true] {
        let status = m.record_result("db", ok).unwrap();
        println!("probe ok={ok} -> {status:?}");
    }
}
