//! Demo CLI: feed utilization samples for a couple of resources through the
//! optimizer and print the sizing advice, most urgent first.

use resource_optimizer_ui::{advise_all, ResourceUsage};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let resources = vec![
        ResourceUsage { name: "cpu".into(), provisioned: 8.0, samples: vec![0.7, 0.92, 0.95] },
        ResourceUsage { name: "memory".into(), provisioned: 16.0, samples: vec![0.05, 0.1, 0.12] },
        ResourceUsage { name: "disk".into(), provisioned: 100.0, samples: vec![0.4, 0.5, 0.55] },
    ];
    for advice in advise_all(&resources)? {
        println!(
            "{}: {:?} (avg {:.2}, peak {:.2})",
            advice.resource, advice.recommendation, advice.avg_utilization, advice.peak_utilization
        );
    }
    Ok(())
}
