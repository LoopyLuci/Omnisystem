//! Demo CLI: summarize and bucket a small dataset.

use visualization_library::{histogram, summarize};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let values = vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0, 12.0, 15.0];
    let summary = summarize(&values)?;
    println!("{summary:?}");
    for bucket in histogram(&values, 4)? {
        println!("{bucket:?}");
    }
    Ok(())
}
