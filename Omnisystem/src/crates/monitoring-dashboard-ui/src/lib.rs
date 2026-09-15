//! Monitoring dashboard: derives per-widget [`Status`] from thresholds and
//! rolls every widget's status up into one overall dashboard status (the
//! worst status wins).

#![warn(missing_docs)]

pub mod error;
pub mod types;

pub use error::{Error, Result};
pub use types::*;

/// Validate a widget's thresholds are well-formed (`warn < critical`).
pub fn validate_widget(widget: &Widget) -> Result<()> {
    if widget.warn_threshold >= widget.critical_threshold {
        return Err(Error::InvalidThresholds { widget: widget.name.clone() });
    }
    Ok(())
}

/// Derive a widget's current status from its latest value and thresholds.
pub fn widget_status(widget: &Widget) -> Result<Status> {
    validate_widget(widget)?;
    Ok(match widget.latest_value {
        None => Status::Unknown,
        Some(v) if v >= widget.critical_threshold => Status::Critical,
        Some(v) if v >= widget.warn_threshold => Status::Warning,
        Some(_) => Status::Ok,
    })
}

/// Roll every widget's status up into one overall status: the most severe
/// status present, where `Critical > Warning > Unknown > Ok`.
pub fn dashboard_status(widgets: &[Widget]) -> Result<Status> {
    if widgets.is_empty() {
        return Err(Error::NoWidgets);
    }
    fn severity(s: Status) -> u8 {
        match s {
            Status::Ok => 0,
            Status::Unknown => 1,
            Status::Warning => 2,
            Status::Critical => 3,
        }
    }
    let mut worst = Status::Ok;
    for w in widgets {
        let s = widget_status(w)?;
        if severity(s) > severity(worst) {
            worst = s;
        }
    }
    Ok(worst)
}

/// Names of widgets at or above `Warning`, sorted, most severe first then
/// by name.
pub fn unhealthy_widgets(widgets: &[Widget]) -> Result<Vec<String>> {
    let mut out: Vec<(Status, String)> = Vec::new();
    for w in widgets {
        let s = widget_status(w)?;
        if matches!(s, Status::Warning | Status::Critical) {
            out.push((s, w.name.clone()));
        }
    }
    out.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    Ok(out.into_iter().map(|(_, name)| name).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn widget(name: &str, value: Option<f64>) -> Widget {
        Widget { name: name.into(), latest_value: value, warn_threshold: 70.0, critical_threshold: 90.0 }
    }

    #[test]
    fn validate_widget_rejects_inverted_thresholds() {
        let w = Widget { name: "cpu".into(), latest_value: None, warn_threshold: 90.0, critical_threshold: 70.0 };
        assert!(matches!(validate_widget(&w), Err(Error::InvalidThresholds { .. })));
    }

    #[test]
    fn widget_status_is_unknown_with_no_data() {
        assert_eq!(widget_status(&widget("cpu", None)).unwrap(), Status::Unknown);
    }

    #[test]
    fn widget_status_is_ok_below_warn() {
        assert_eq!(widget_status(&widget("cpu", Some(50.0))).unwrap(), Status::Ok);
    }

    #[test]
    fn widget_status_is_warning_at_warn_threshold() {
        assert_eq!(widget_status(&widget("cpu", Some(70.0))).unwrap(), Status::Warning);
    }

    #[test]
    fn widget_status_is_critical_at_critical_threshold() {
        assert_eq!(widget_status(&widget("cpu", Some(95.0))).unwrap(), Status::Critical);
    }

    #[test]
    fn dashboard_status_is_worst_of_all_widgets() {
        let widgets = vec![widget("cpu", Some(50.0)), widget("mem", Some(95.0))];
        assert_eq!(dashboard_status(&widgets).unwrap(), Status::Critical);
    }

    #[test]
    fn dashboard_status_errors_with_no_widgets() {
        assert_eq!(dashboard_status(&[]), Err(Error::NoWidgets));
    }

    #[test]
    fn dashboard_status_all_ok_is_ok() {
        let widgets = vec![widget("cpu", Some(10.0)), widget("mem", Some(20.0))];
        assert_eq!(dashboard_status(&widgets).unwrap(), Status::Ok);
    }

    #[test]
    fn unhealthy_widgets_lists_warning_and_critical_only_most_severe_first() {
        let widgets = vec![widget("cpu", Some(10.0)), widget("mem", Some(75.0)), widget("disk", Some(95.0))];
        assert_eq!(unhealthy_widgets(&widgets).unwrap(), vec!["disk".to_string(), "mem".to_string()]);
    }
}
