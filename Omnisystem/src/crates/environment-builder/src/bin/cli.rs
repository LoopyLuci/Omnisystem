//! Demo CLI: layer three environments, validate, and print the merged
//! result plus a diff against the previous release.

use environment_builder::{diff, merge_layers, validate_all, Layer, VarSpec};

fn main() {
    let layers = vec![
        Layer::new("base").with("LOG_LEVEL", "info").with("REGION", "us-east"),
        Layer::new("staging").with("REGION", "us-west"),
    ];
    let merged = merge_layers(&layers);
    let schema = vec![VarSpec { name: "LOG_LEVEL".into(), required: true, allowed_values: vec![] }];
    for err in validate_all(&merged, &schema) {
        eprintln!("validation error: {err}");
    }
    for (k, v) in &merged {
        println!("{k}={v}");
    }
    let previous = merge_layers(&[Layer::new("base").with("LOG_LEVEL", "info").with("REGION", "us-east")]);
    for d in diff(&previous, &merged) {
        println!("{d:?}");
    }
}
