//! Keyframe and easing types.

use serde::{Deserialize, Serialize};

/// A named easing function applied between two keyframes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Easing {
    /// Constant rate of change.
    Linear,
    /// Starts slow, accelerates.
    EaseIn,
    /// Starts fast, decelerates.
    EaseOut,
    /// Slow, fast, slow.
    EaseInOut,
}

impl Easing {
    /// Apply this easing to a normalized time `t` in `[0, 1]`, returning an
    /// eased progress also in `[0, 1]`.
    pub fn apply(&self, t: f64) -> f64 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Easing::Linear => t,
            Easing::EaseIn => t * t,
            Easing::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
            Easing::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }
        }
    }
}

/// A single keyframe: a value at a point in time, with the easing to use
/// on the segment leading *into* it from the previous keyframe.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Keyframe {
    /// Time in seconds, measured from the start of the timeline.
    pub time: f64,
    /// The animated value at this time.
    pub value: f64,
    /// Easing applied on the segment ending at this keyframe.
    pub easing: Easing,
}

impl Keyframe {
    /// Construct a new keyframe.
    pub fn new(time: f64, value: f64, easing: Easing) -> Self {
        Self { time, value, easing }
    }
}
