//! Keyframe-based animation timeline: sample an interpolated value at any
//! point in time, with per-segment easing. Deliberately renderer-agnostic —
//! this crate computes the numbers a UI layer would apply to a property.

#![warn(missing_docs)]

pub mod error;
pub mod types;

pub use error::{Error, Result};
pub use types::*;

/// An ordered sequence of keyframes describing how one value changes over
/// time.
#[derive(Debug, Clone, Default)]
pub struct Timeline {
    keyframes: Vec<Keyframe>,
}

impl Timeline {
    /// Build a timeline from keyframes, validating that they are given in
    /// non-decreasing time order.
    pub fn new(keyframes: Vec<Keyframe>) -> Result<Self> {
        if keyframes.is_empty() {
            return Err(Error::NoKeyframes);
        }
        for pair in keyframes.windows(2) {
            if pair[1].time < pair[0].time {
                return Err(Error::UnorderedKeyframes(pair[0].time, pair[1].time));
            }
        }
        Ok(Self { keyframes })
    }

    /// Total duration of the timeline, from the first to the last keyframe.
    pub fn duration(&self) -> f64 {
        self.keyframes.last().unwrap().time - self.keyframes.first().unwrap().time
    }

    /// Sample the interpolated value at time `t`. Times before the first
    /// keyframe clamp to its value; times after the last clamp likewise.
    pub fn sample(&self, t: f64) -> f64 {
        let first = self.keyframes.first().unwrap();
        let last = self.keyframes.last().unwrap();
        if t <= first.time {
            return first.value;
        }
        if t >= last.time {
            return last.value;
        }
        // Find the segment [a, b] containing t.
        let idx = self.keyframes.partition_point(|k| k.time <= t);
        let a = &self.keyframes[idx - 1];
        let b = &self.keyframes[idx];
        let span = b.time - a.time;
        let local_t = if span > 0.0 { (t - a.time) / span } else { 1.0 };
        let eased = b.easing.apply(local_t);
        a.value + (b.value - a.value) * eased
    }

    /// Sample `count` evenly spaced points across the timeline's duration
    /// (inclusive of both endpoints), useful for pre-baking a frame table.
    pub fn bake(&self, count: usize) -> Result<Vec<(f64, f64)>> {
        if count < 2 {
            return Err(Error::InvalidDuration(count as f64));
        }
        let start = self.keyframes.first().unwrap().time;
        let dur = self.duration();
        let mut out = Vec::with_capacity(count);
        for i in 0..count {
            let t = start + dur * (i as f64 / (count - 1) as f64);
            out.push((t, self.sample(t)));
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_timeline() {
        assert_eq!(Timeline::new(vec![]).unwrap_err(), Error::NoKeyframes);
    }

    #[test]
    fn rejects_unordered_keyframes() {
        let kfs = vec![
            Keyframe::new(1.0, 0.0, Easing::Linear),
            Keyframe::new(0.5, 1.0, Easing::Linear),
        ];
        assert!(matches!(Timeline::new(kfs), Err(Error::UnorderedKeyframes(_, _))));
    }

    #[test]
    fn linear_interpolation_is_exact_at_midpoint() {
        let t = Timeline::new(vec![
            Keyframe::new(0.0, 0.0, Easing::Linear),
            Keyframe::new(10.0, 100.0, Easing::Linear),
        ])
        .unwrap();
        assert_eq!(t.sample(5.0), 50.0);
    }

    #[test]
    fn samples_clamp_before_and_after_range() {
        let t = Timeline::new(vec![
            Keyframe::new(1.0, 5.0, Easing::Linear),
            Keyframe::new(2.0, 15.0, Easing::Linear),
        ])
        .unwrap();
        assert_eq!(t.sample(-5.0), 5.0);
        assert_eq!(t.sample(100.0), 15.0);
    }

    #[test]
    fn multi_segment_picks_correct_segment() {
        let t = Timeline::new(vec![
            Keyframe::new(0.0, 0.0, Easing::Linear),
            Keyframe::new(1.0, 10.0, Easing::Linear),
            Keyframe::new(2.0, 0.0, Easing::Linear),
        ])
        .unwrap();
        assert_eq!(t.sample(0.5), 5.0);
        assert_eq!(t.sample(1.5), 5.0);
    }

    #[test]
    fn ease_in_starts_slower_than_linear() {
        assert!(Easing::EaseIn.apply(0.25) < 0.25);
    }

    #[test]
    fn ease_out_starts_faster_than_linear() {
        assert!(Easing::EaseOut.apply(0.25) > 0.25);
    }

    #[test]
    fn ease_in_out_is_symmetric_about_midpoint() {
        let a = Easing::EaseInOut.apply(0.25);
        let b = Easing::EaseInOut.apply(0.75);
        assert!((a + b - 1.0).abs() < 1e-9);
    }

    #[test]
    fn easing_endpoints_are_fixed() {
        for e in [Easing::Linear, Easing::EaseIn, Easing::EaseOut, Easing::EaseInOut] {
            assert!((e.apply(0.0) - 0.0).abs() < 1e-9);
            assert!((e.apply(1.0) - 1.0).abs() < 1e-9);
        }
    }

    #[test]
    fn duration_is_span_of_first_and_last() {
        let t = Timeline::new(vec![
            Keyframe::new(2.0, 0.0, Easing::Linear),
            Keyframe::new(7.5, 1.0, Easing::Linear),
        ])
        .unwrap();
        assert_eq!(t.duration(), 5.5);
    }

    #[test]
    fn bake_produces_requested_sample_count() {
        let t = Timeline::new(vec![
            Keyframe::new(0.0, 0.0, Easing::Linear),
            Keyframe::new(1.0, 1.0, Easing::Linear),
        ])
        .unwrap();
        let baked = t.bake(11).unwrap();
        assert_eq!(baked.len(), 11);
        assert_eq!(baked.first().unwrap().1, 0.0);
        assert_eq!(baked.last().unwrap().1, 1.0);
    }

    #[test]
    fn bake_rejects_too_few_samples() {
        let t = Timeline::new(vec![Keyframe::new(0.0, 0.0, Easing::Linear)]).unwrap();
        assert!(t.bake(1).is_err());
    }
}
