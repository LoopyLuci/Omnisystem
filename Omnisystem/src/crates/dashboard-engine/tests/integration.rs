//! End-to-end: auto-pack a realistic widget set, then re-validate the
//! result to confirm the packer's own output always passes its validator.

use dashboard_engine::{Grid, WidgetSpec};

#[test]
fn auto_packed_layout_is_always_valid() {
    let grid = Grid::new(12);
    let widgets = vec![
        WidgetSpec::new("kpi-revenue", 3, 2),
        WidgetSpec::new("kpi-users", 3, 2),
        WidgetSpec::new("kpi-churn", 3, 2),
        WidgetSpec::new("kpi-nps", 3, 2),
        WidgetSpec::new("revenue-trend", 8, 4),
        WidgetSpec::new("top-accounts", 4, 4),
        WidgetSpec::new("activity-log", 12, 6),
    ];

    let placed = grid.auto_pack(&widgets).expect("widgets fit within grid width");
    assert_eq!(placed.len(), widgets.len());
    grid.validate(&placed).expect("packer output must never overlap or overflow");

    let height = grid.content_height(&placed);
    assert!(height > 0);
    // Every widget's bottom edge must be within the reported content height.
    for p in &placed {
        assert!(p.row + p.height <= height);
    }
}
