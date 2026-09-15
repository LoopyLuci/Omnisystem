//! Placement geometry types.

use serde::{Deserialize, Serialize};

/// An axis-aligned rectangle in viewport pixel coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    /// Left edge.
    pub x: f64,
    /// Top edge.
    pub y: f64,
    /// Width.
    pub w: f64,
    /// Height.
    pub h: f64,
}

impl Rect {
    /// Construct a new rectangle.
    pub fn new(x: f64, y: f64, w: f64, h: f64) -> Self {
        Self { x, y, w, h }
    }

    /// Right edge (`x + w`).
    pub fn right(&self) -> f64 {
        self.x + self.w
    }

    /// Bottom edge (`y + h`).
    pub fn bottom(&self) -> f64 {
        self.y + self.h
    }
}

/// Which side of the anchor element the popover is placed on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    /// Above the anchor.
    Top,
    /// Below the anchor.
    Bottom,
    /// To the left of the anchor.
    Left,
    /// To the right of the anchor.
    Right,
}

impl Side {
    /// The side directly opposite this one.
    pub fn opposite(&self) -> Side {
        match self {
            Side::Top => Side::Bottom,
            Side::Bottom => Side::Top,
            Side::Left => Side::Right,
            Side::Right => Side::Left,
        }
    }
}

/// A computed popover placement: which side it ended up on, and its
/// top-left position.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Placement {
    /// The side actually used (may differ from the preferred side if it
    /// didn't fit and flipping was needed).
    pub side: Side,
    /// Final top-left position of the popover.
    pub x: f64,
    /// Final top-left position of the popover.
    pub y: f64,
}
