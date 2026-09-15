//! Chart data model.

use serde::{Deserialize, Serialize};

/// A single (x, y) sample in a series.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DataPoint {
    /// Horizontal coordinate (category index or numeric x).
    pub x: f64,
    /// Vertical coordinate (the measured value).
    pub y: f64,
}

impl DataPoint {
    /// Construct a new data point.
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

/// The visual encoding a chart uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChartType {
    /// Continuous line connecting points in x order.
    Line,
    /// Discrete bars, one per point.
    Bar,
    /// Proportional wedges of a whole.
    Pie,
    /// Unconnected points.
    Scatter,
}

/// A named collection of points sharing one visual style.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Series {
    /// Series label, shown in the legend.
    pub name: String,
    /// The data points, not required to be sorted by the caller.
    pub points: Vec<DataPoint>,
}

impl Series {
    /// Construct a new, empty series.
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), points: Vec::new() }
    }

    /// Append a point.
    pub fn push(&mut self, x: f64, y: f64) {
        self.points.push(DataPoint::new(x, y));
    }

    /// The minimum and maximum `y` values in this series, if any.
    pub fn y_extent(&self) -> Option<(f64, f64)> {
        if self.points.is_empty() {
            return None;
        }
        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;
        for p in &self.points {
            min = min.min(p.y);
            max = max.max(p.y);
        }
        Some((min, max))
    }

    /// Sum of all `y` values.
    pub fn sum(&self) -> f64 {
        self.points.iter().map(|p| p.y).sum()
    }

    /// Arithmetic mean of all `y` values, if any points exist.
    pub fn mean(&self) -> Option<f64> {
        if self.points.is_empty() {
            None
        } else {
            Some(self.sum() / self.points.len() as f64)
        }
    }
}

/// A chart: one or more series rendered with a shared visual encoding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chart {
    /// Chart title.
    pub title: String,
    /// Which visual encoding to use.
    pub chart_type: ChartType,
    /// The series to plot.
    pub series: Vec<Series>,
}

impl Chart {
    /// Construct a new, empty chart.
    pub fn new(title: impl Into<String>, chart_type: ChartType) -> Self {
        Self { title: title.into(), chart_type, series: Vec::new() }
    }

    /// Add a series to the chart.
    pub fn add_series(&mut self, series: Series) {
        self.series.push(series);
    }
}

/// One wedge of a pie chart, with its computed angular span.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PieSlice {
    /// Raw value this slice represents.
    pub value: f64,
    /// Fraction of the whole, in `[0, 1]`.
    pub fraction: f64,
    /// Start angle in degrees, measured clockwise from 12 o'clock.
    pub start_angle: f64,
    /// End angle in degrees.
    pub end_angle: f64,
}
