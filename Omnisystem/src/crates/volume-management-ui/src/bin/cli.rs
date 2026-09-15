//! Demo CLI: plan volume placement across two disks.

use volume_management_ui::{plan_allocation, Disk, Volume};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let volumes = vec![Volume { name: "data-01".into(), size_gb: 60.0 }, Volume { name: "logs-01".into(), size_gb: 20.0 }];
    let disks = vec![
        Disk { name: "disk1".into(), capacity_gb: 100.0, used_gb: 0.0 },
        Disk { name: "disk2".into(), capacity_gb: 100.0, used_gb: 0.0 },
    ];
    for p in plan_allocation(&volumes, &disks)? {
        println!("{} -> {}", p.volume, p.disk);
    }
    Ok(())
}
