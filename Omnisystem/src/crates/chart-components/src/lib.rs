//! Chart data preparation: domain computation, "nice" axis tick generation,
//! and pie-slice angle computation. Deliberately framework/renderer-agnostic
//! — this crate prepares the numbers a renderer would draw, not pixels.

#![warn(missing_docs)]

pub mod error;
pub mod types;

pub use error::{Error, Result};
pub use types::*;

impl Chart {
    /// The combined `y` domain (min, max) across every series, ignoring
    /// series that are empty.
    pub fn domain(&self) -> Result<(f64, f64)> {
        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;
        let mut any = false;
        for s in &self.series {
            if let Some((smin, smax)) = s.y_extent() {
                min = min.min(smin);
                max = max.max(smax);
                any = true;
            }
        }
        if !any {
            return Err(Error::EmptySeries(
                self.series.first().map(|s| s.name.clone()).unwrap_or_default(),
            ));
        }
        Ok((min, max))
    }

    /// Compute pie slices from the first series' `y` values, in point order.
    /// Angles run clockwise starting at 0 degrees (12 o'clock).
    pub fn to_pie_slices(&self) -> Result<Vec<PieSlice>> {
        let series = self.series.first().ok_or_else(|| Error::EmptySeries("".into()))?;
        if series.points.is_empty() {
            return Err(Error::EmptySeries(series.name.clone()));
        }
        let total: f64 = series.points.iter().map(|p| p.y).sum();
        if total <= 0.0 {
            return Err(Error::NonPositiveTotal);
        }
        let mut angle = 0.0;
        let mut slices = Vec::with_capacity(series.points.len());
        for p in &series.points {
            let fraction = p.y / total;
            let span = fraction * 360.0;
            slices.push(PieSlice {
                value: p.y,
                fraction,
                start_angle: angle,
                end_angle: angle + span,
            });
            angle += span;
        }
        Ok(slices)
    }
}

/// A linear scale mapping a data domain to a pixel range.
#[derive(Debug, Clone, Copy)]
pub struct LinearScale {
    domain_min: f64,
    domain_max: f64,
    range_min: f64,
    range_max: f64,
}

impl LinearScale {
    /// Build a scale from `domain` (data space) to `range` (pixel space).
    pub fn new(domain: (f64, f64), range: (f64, f64)) -> Result<Self> {
        if domain.0 >= domain.1 {
            return Err(Error::InvalidDomain(domain.0, domain.1));
        }
        Ok(Self { domain_min: domain.0, domain_max: domain.1, range_min: range.0, range_max: range.1 })
    }

    /// Map a data value into range space.
    pub fn map(&self, value: f64) -> f64 {
        let t = (value - self.domain_min) / (self.domain_max - self.domain_min);
        self.range_min + t * (self.range_max - self.range_min)
    }

    /// Map a range-space value back into the data domain.
    pub fn invert(&self, position: f64) -> f64 {
        let t = (position - self.range_min) / (self.range_max - self.range_min);
        self.domain_min + t * (self.domain_max - self.domain_min)
    }
}

/// Generate "nice" round-number axis ticks spanning `(min, max)`, targeting
/// roughly `count` ticks (a classic D3-style nice-number algorithm).
pub fn nice_ticks(min: f64, max: f64, count: usize) -> Result<Vec<f64>> {
    if min >= max || count == 0 {
        return Err(Error::InvalidDomain(min, max));
    }
    let raw_step = (max - min) / count as f64;
    let magnitude = 10f64.powf(raw_step.log10().floor());
    let residual = raw_step / magnitude;
    let nice_residual = if residual < 1.5 {
        1.0
    } else if residual < 3.0 {
        2.0
    } else if residual < 7.0 {
        5.0
    } else {
        10.0
    };
    let step = nice_residual * magnitude;

    let start = (min / step).floor() * step;
    let mut ticks = Vec::new();
    let mut t = start;
    // Guard against float drift producing an unbounded loop.
    let max_ticks = count * 4 + 4;
    while t < max && ticks.len() < max_ticks {
        ticks.push((t / step).round() * step);
        t += step;
    }
    // Ensure the final tick covers (is >=) max, per this function's contract.
    let last_covers_max = ticks.last().is_some_and(|&t| t >= max);
    if !last_covers_max && ticks.len() < max_ticks {
        ticks.push((t / step).round() * step);
    }
    Ok(ticks)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_chart() -> Chart {
        let mut chart = Chart::new("Revenue", ChartType::Bar);
        let mut s = Series::new("Q1-Q4");
        s.push(0.0, 10.0);
        s.push(1.0, 25.0);
        s.push(2.0, 5.0);
        s.push(3.0, 60.0);
        chart.add_series(s);
        chart
    }

    #[test]
    fn domain_spans_all_series() {
        let chart = sample_chart();
        assert_eq!(chart.domain().unwrap(), (5.0, 60.0));
    }

    #[test]
    fn domain_errors_on_empty_series() {
        let chart = Chart::new("Empty", ChartType::Line);
        assert_eq!(chart.domain(), Err(Error::EmptySeries(String::new())));
    }

    #[test]
    fn pie_slices_sum_to_360_degrees() {
        let chart = sample_chart();
        let slices = chart.to_pie_slices().unwrap();
        assert_eq!(slices.len(), 4);
        assert!((slices.last().unwrap().end_angle - 360.0).abs() < 1e-9);
    }

    #[test]
    fn pie_slices_are_contiguous() {
        let chart = sample_chart();
        let slices = chart.to_pie_slices().unwrap();
        for pair in slices.windows(2) {
            assert!((pair[0].end_angle - pair[1].start_angle).abs() < 1e-9);
        }
    }

    #[test]
    fn pie_rejects_non_positive_total() {
        let mut chart = Chart::new("Bad", ChartType::Pie);
        let mut s = Series::new("s");
        s.push(0.0, -5.0);
        s.push(1.0, 5.0);
        chart.add_series(s);
        assert_eq!(chart.to_pie_slices(), Err(Error::NonPositiveTotal));
    }

    #[test]
    fn linear_scale_maps_domain_to_range() {
        let scale = LinearScale::new((0.0, 100.0), (0.0, 500.0)).unwrap();
        assert_eq!(scale.map(0.0), 0.0);
        assert_eq!(scale.map(100.0), 500.0);
        assert_eq!(scale.map(50.0), 250.0);
    }

    #[test]
    fn linear_scale_invert_round_trips() {
        let scale = LinearScale::new((10.0, 20.0), (0.0, 1.0)).unwrap();
        let mapped = scale.map(15.0);
        assert!((scale.invert(mapped) - 15.0).abs() < 1e-9);
    }

    #[test]
    fn linear_scale_rejects_degenerate_domain() {
        assert!(LinearScale::new((5.0, 5.0), (0.0, 1.0)).is_err());
    }

    #[test]
    fn nice_ticks_bracket_the_domain() {
        let ticks = nice_ticks(3.0, 27.0, 5).unwrap();
        assert!(*ticks.first().unwrap() <= 3.0);
        assert!(*ticks.last().unwrap() >= 27.0);
    }

    #[test]
    fn nice_ticks_are_evenly_spaced() {
        let ticks = nice_ticks(0.0, 100.0, 5).unwrap();
        let step = ticks[1] - ticks[0];
        for pair in ticks.windows(2) {
            assert!((pair[1] - pair[0] - step).abs() < 1e-9);
        }
    }

    #[test]
    fn nice_ticks_rejects_bad_domain() {
        assert!(nice_ticks(10.0, 5.0, 5).is_err());
        assert!(nice_ticks(0.0, 10.0, 0).is_err());
    }

    #[test]
    fn series_mean_and_sum() {
        let mut s = Series::new("x");
        s.push(0.0, 2.0);
        s.push(1.0, 4.0);
        s.push(2.0, 6.0);
        assert_eq!(s.sum(), 12.0);
        assert_eq!(s.mean(), Some(4.0));
    }
}
