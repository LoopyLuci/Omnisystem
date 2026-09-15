//! Demo CLI: roll up widget statuses into one dashboard status.

use monitoring_dashboard_ui::{dashboard_status, unhealthy_widgets, Widget};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let widgets = vec![
        Widget { name: "cpu".into(), latest_value: Some(45.0), warn_threshold: 70.0, critical_threshold: 90.0 },
        Widget { name: "disk".into(), latest_value: Some(95.0), warn_threshold: 70.0, critical_threshold: 90.0 },
    ];
    println!("dashboard status: {:?}", dashboard_status(&widgets)?);
    println!("unhealthy: {:?}", unhealthy_widgets(&widgets)?);
    Ok(())
}
