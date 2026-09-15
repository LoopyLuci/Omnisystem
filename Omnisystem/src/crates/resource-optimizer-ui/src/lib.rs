//! Resource optimizer UI: turns raw utilization samples into right-sizing
//! recommendations — scale up when peak utilization is consistently near
//! capacity, scale down when it's consistently low, otherwise keep as is.
//! Thresholds are fixed constants so recommendations are reproducible.

#![warn(missing_docs)]

pub mod error;
pub mod types;

pub use error::{Error, Result};
pub use types::*;

/// Utilization at or above this fraction is considered "near capacity".
pub const SCALE_UP_THRESHOLD: f64 = 0.85;
/// Utilization at or below this fraction is considered "underused".
pub const SCALE_DOWN_THRESHOLD: f64 = 0.20;

/// Validate that every sample for a resource is within `0.0..=1.0`.
pub fn validate_samples(usage: &ResourceUsage) -> Result<()> {
    for &s in &usage.samples {
        if !(0.0..=1.0).contains(&s) {
            return Err(Error::UtilizationOutOfRange { resource: usage.name.clone(), value: s });
        }
    }
    Ok(())
}

/// Produce sizing advice for one resource from its utilization samples.
pub fn advise(usage: &ResourceUsage) -> Result<SizingAdvice> {
    validate_samples(usage)?;
    if usage.samples.is_empty() {
        return Err(Error::NoSamples(usage.name.clone()));
    }
    let avg = usage.samples.iter().sum::<f64>() / usage.samples.len() as f64;
    let peak = usage.samples.iter().cloned().fold(f64::MIN, f64::max);
    let recommendation = if peak >= SCALE_UP_THRESHOLD {
        Recommendation::ScaleUp
    } else if peak <= SCALE_DOWN_THRESHOLD {
        Recommendation::ScaleDown
    } else {
        Recommendation::Keep
    };
    Ok(SizingAdvice { resource: usage.name.clone(), avg_utilization: avg, peak_utilization: peak, recommendation })
}

/// Produce advice for every resource, skipping (not failing on) resources
/// with no samples, and sort the result so the most urgent (`ScaleUp`)
/// recommendations surface first.
pub fn advise_all(resources: &[ResourceUsage]) -> Result<Vec<SizingAdvice>> {
    let mut out = Vec::new();
    for r in resources {
        if r.samples.is_empty() {
            continue;
        }
        out.push(advise(r)?);
    }
    out.sort_by(|a, b| priority(a.recommendation).cmp(&priority(b.recommendation)));
    Ok(out)
}

fn priority(r: Recommendation) -> u8 {
    match r {
        Recommendation::ScaleUp => 0,
        Recommendation::ScaleDown => 1,
        Recommendation::Keep => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn usage(name: &str, samples: Vec<f64>) -> ResourceUsage {
        ResourceUsage { name: name.into(), provisioned: 4.0, samples }
    }

    #[test]
    fn validate_rejects_out_of_range_sample() {
        let u = usage("cpu", vec![1.5]);
        assert!(matches!(validate_samples(&u), Err(Error::UtilizationOutOfRange { .. })));
    }

    #[test]
    fn validate_accepts_boundary_values() {
        let u = usage("cpu", vec![0.0, 1.0]);
        assert!(validate_samples(&u).is_ok());
    }

    #[test]
    fn advise_recommends_scale_up_near_capacity() {
        let u = usage("cpu", vec![0.7, 0.9, 0.95]);
        let advice = advise(&u).unwrap();
        assert_eq!(advice.recommendation, Recommendation::ScaleUp);
        assert_eq!(advice.peak_utilization, 0.95);
    }

    #[test]
    fn advise_recommends_scale_down_when_underused() {
        let u = usage("cpu", vec![0.05, 0.1, 0.15]);
        assert_eq!(advise(&u).unwrap().recommendation, Recommendation::ScaleDown);
    }

    #[test]
    fn advise_recommends_keep_in_healthy_band() {
        let u = usage("cpu", vec![0.4, 0.5, 0.6]);
        assert_eq!(advise(&u).unwrap().recommendation, Recommendation::Keep);
    }

    #[test]
    fn advise_computes_correct_average() {
        let u = usage("cpu", vec![0.2, 0.4, 0.6]);
        let advice = advise(&u).unwrap();
        assert!((advice.avg_utilization - 0.4).abs() < 1e-9);
    }

    #[test]
    fn advise_errors_on_no_samples() {
        let u = usage("cpu", vec![]);
        assert_eq!(advise(&u), Err(Error::NoSamples("cpu".into())));
    }

    #[test]
    fn advise_all_skips_empty_resources_and_sorts_by_urgency() {
        let resources = vec![
            usage("mem", vec![0.5]),
            usage("disk", vec![]),
            usage("cpu", vec![0.9]),
        ];
        let advices = advise_all(&resources).unwrap();
        assert_eq!(advices.len(), 2);
        assert_eq!(advices[0].resource, "cpu");
        assert_eq!(advices[0].recommendation, Recommendation::ScaleUp);
    }

    #[test]
    fn advise_all_propagates_validation_error() {
        let resources = vec![usage("cpu", vec![2.0])];
        assert!(advise_all(&resources).is_err());
    }
}
