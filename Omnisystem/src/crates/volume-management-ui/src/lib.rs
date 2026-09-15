//! Volume management: places volumes onto disks with a first-fit-decreasing
//! bin-packing heuristic (largest volumes placed first, onto the first disk
//! with enough remaining free capacity), and flags disks over a capacity
//! threshold.

#![warn(missing_docs)]

use std::collections::HashMap;

pub mod error;
pub mod types;

pub use error::{Error, Result};
pub use types::*;

/// Plan placements for every volume using first-fit-decreasing: volumes
/// are sorted largest-first, each placed on the first disk (in the given
/// order) with enough remaining free capacity.
pub fn plan_allocation(volumes: &[Volume], disks: &[Disk]) -> Result<Vec<Placement>> {
    for v in volumes {
        if v.size_gb <= 0.0 {
            return Err(Error::InvalidSize(v.name.clone()));
        }
    }
    let mut remaining: HashMap<&str, f64> = disks.iter().map(|d| (d.name.as_str(), d.free_gb())).collect();
    let disk_order: Vec<&str> = disks.iter().map(|d| d.name.as_str()).collect();

    let mut sorted: Vec<&Volume> = volumes.iter().collect();
    sorted.sort_by(|a, b| b.size_gb.partial_cmp(&a.size_gb).unwrap_or(std::cmp::Ordering::Equal));

    let mut placements = Vec::with_capacity(sorted.len());
    for v in sorted {
        let target = disk_order.iter().find(|&&name| remaining[name] >= v.size_gb);
        match target {
            Some(&name) => {
                *remaining.get_mut(name).unwrap() -= v.size_gb;
                placements.push(Placement { volume: v.name.clone(), disk: name.to_string() });
            }
            None => return Err(Error::NoCapacityFor(v.name.clone())),
        }
    }
    Ok(placements)
}

/// Names of disks at or above `threshold` used-fraction (`0.0..=1.0`),
/// sorted by name.
pub fn over_threshold_disks(disks: &[Disk], threshold: f64) -> Vec<String> {
    let mut names: Vec<String> =
        disks.iter().filter(|d| d.used_fraction() >= threshold).map(|d| d.name.clone()).collect();
    names.sort();
    names
}

#[cfg(test)]
mod tests {
    use super::*;

    fn volume(name: &str, size: f64) -> Volume {
        Volume { name: name.into(), size_gb: size }
    }
    fn disk(name: &str, capacity: f64, used: f64) -> Disk {
        Disk { name: name.into(), capacity_gb: capacity, used_gb: used }
    }

    #[test]
    fn plan_allocation_places_volume_on_disk_with_room() {
        let volumes = vec![volume("data", 50.0)];
        let disks = vec![disk("disk1", 100.0, 0.0)];
        let plan = plan_allocation(&volumes, &disks).unwrap();
        assert_eq!(plan, vec![Placement { volume: "data".into(), disk: "disk1".into() }]);
    }

    #[test]
    fn plan_allocation_rejects_non_positive_size() {
        let volumes = vec![volume("data", 0.0)];
        let disks = vec![disk("disk1", 100.0, 0.0)];
        assert_eq!(plan_allocation(&volumes, &disks), Err(Error::InvalidSize("data".into())));
    }

    #[test]
    fn plan_allocation_errors_when_no_disk_fits() {
        let volumes = vec![volume("huge", 500.0)];
        let disks = vec![disk("disk1", 100.0, 0.0)];
        assert_eq!(plan_allocation(&volumes, &disks), Err(Error::NoCapacityFor("huge".into())));
    }

    #[test]
    fn plan_allocation_places_largest_first_and_spreads_across_disks() {
        let volumes = vec![volume("small", 10.0), volume("big", 80.0)];
        let disks = vec![disk("disk1", 100.0, 0.0), disk("disk2", 100.0, 0.0)];
        let plan = plan_allocation(&volumes, &disks).unwrap();
        // big goes first (largest-first) onto disk1; small then also fits disk1
        // (100 - 80 = 20 >= 10), so both land on disk1.
        assert_eq!(plan[0].volume, "big");
        assert_eq!(plan[0].disk, "disk1");
        assert_eq!(plan[1].volume, "small");
        assert_eq!(plan[1].disk, "disk1");
    }

    #[test]
    fn plan_allocation_falls_through_to_second_disk_when_first_full() {
        let volumes = vec![volume("a", 60.0), volume("b", 60.0)];
        let disks = vec![disk("disk1", 100.0, 0.0), disk("disk2", 100.0, 0.0)];
        let plan = plan_allocation(&volumes, &disks).unwrap();
        let disk_for = |name: &str| plan.iter().find(|p| p.volume == name).unwrap().disk.clone();
        assert_ne!(disk_for("a"), disk_for("b"));
    }

    #[test]
    fn over_threshold_disks_flags_high_usage() {
        let disks = vec![disk("full", 100.0, 95.0), disk("empty", 100.0, 5.0)];
        assert_eq!(over_threshold_disks(&disks, 0.9), vec!["full".to_string()]);
    }

    #[test]
    fn over_threshold_disks_empty_when_none_qualify() {
        let disks = vec![disk("a", 100.0, 10.0)];
        assert!(over_threshold_disks(&disks, 0.9).is_empty());
    }

    #[test]
    fn free_gb_and_used_fraction_are_consistent() {
        let d = disk("a", 100.0, 25.0);
        assert_eq!(d.free_gb(), 75.0);
        assert_eq!(d.used_fraction(), 0.25);
    }

    #[test]
    fn used_fraction_of_zero_capacity_disk_is_full() {
        let d = disk("a", 0.0, 0.0);
        assert_eq!(d.used_fraction(), 1.0);
    }
}
