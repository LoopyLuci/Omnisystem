//! End-to-end: simulate a scroll sequence over a variable-height feed and
//! check the visible range advances and the near-bottom fetch signal fires
//! at the expected point.

use infinite_scroll_component::Virtualizer;

#[test]
fn scroll_sequence_advances_range_and_signals_near_bottom() {
    // 50 posts, alternating short/tall (simulating a social feed).
    let heights: Vec<f64> = (0..50).map(|i| if i % 3 == 0 { 200.0 } else { 80.0 }).collect();
    let v = Virtualizer::new(&heights);
    let total = v.total_height();

    let top_range = v.visible_range(0.0, 800.0, 1).unwrap();
    assert_eq!(top_range.start_index, 0);

    let mid_range = v.visible_range(total / 2.0, 800.0, 1).unwrap();
    assert!(mid_range.start_index > top_range.start_index);

    let bottom_scroll = total - 850.0;
    let bottom_range = v.visible_range(bottom_scroll, 800.0, 1).unwrap();
    assert_eq!(bottom_range.end_index, v.len() - 1);

    assert!(v.near_bottom(bottom_scroll, 800.0, 100.0));
    assert!(!v.near_bottom(0.0, 800.0, 100.0));
}
