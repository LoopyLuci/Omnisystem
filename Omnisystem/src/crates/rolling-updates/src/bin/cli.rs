//! CLI demo: roll a 10-instance fleet forward with max_unavailable=3.

use rolling_updates::{BatchResult, Manager};

fn main() {
    let manager = Manager::new(10, 3).expect("valid config");
    loop {
        let size = match manager.start_batch() {
            Ok(size) => size,
            Err(e) => {
                println!("rollout finished: {e}");
                break;
            }
        };
        println!("took {size} instance(s) out of service");
        match manager.complete_batch().expect("batch was started") {
            BatchResult::Remaining(n) => println!("batch updated; {n} remaining"),
            BatchResult::Done => {
                println!("rollout complete: {}/{}", manager.updated_count(), 10);
                break;
            }
        }
    }
}
