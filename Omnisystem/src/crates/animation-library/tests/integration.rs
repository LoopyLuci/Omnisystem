//! End-to-end: a three-keyframe timeline sampled and baked, checking
//! monotonic-ish continuity and endpoint fidelity.

use animation_library::{Easing, Keyframe, Timeline};

#[test]
fn bounce_timeline_samples_are_continuous_and_bounded() {
    let timeline = Timeline::new(vec![
        Keyframe::new(0.0, 0.0, Easing::Linear),
        Keyframe::new(1.0, 100.0, Easing::EaseOut),
        Keyframe::new(2.0, 0.0, Easing::EaseIn),
    ])
    .expect("ordered keyframes");

    let baked = timeline.bake(21).expect("valid sample count");
    assert_eq!(baked.first().unwrap().1, 0.0);
    assert_eq!(baked.last().unwrap().1, 0.0);
    for (_, v) in &baked {
        assert!((0.0..=100.0).contains(v), "value {v} escaped expected bounds");
    }

    // Peak should land near the middle keyframe.
    let (peak_t, peak_v) = baked
        .iter()
        .copied()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .unwrap();
    assert!((peak_t - 1.0).abs() < 0.15, "peak at {peak_t}, expected near t=1.0");
    assert!(peak_v > 90.0);
}
