//! Dashboard-facing input/output shapes.
//!
//! `security-console-ui` presents no security logic of its own — it does
//! not evaluate RBAC, scan containers, or run static analysis. It shapes
//! whatever those crates' outputs are (modeled here as a generic
//! `SourceEvent`, standing in for `SecurityFinding`/`Finding`/etc. from the
//! other crates in this cluster) into the aggregated, sorted, paginated
//! form a dashboard view actually renders.

use serde::{Deserialize, Serialize};

/// A minimal, source-agnostic stand-in for the richer finding/event types
/// produced by the domain crates (`omnisystem-security-integration`,
/// `container-security-platform`, `security-analyzer`, ...). The console
/// only needs enough fields to sort, group, and paginate for display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceEvent {
    /// Stable id from the originating system.
    pub id: String,
    /// Originating system name, e.g. "container-security-platform".
    pub source: String,
    /// Severity, 0 (lowest) to 4 (highest) — a numeric scale so this
    /// crate stays decoupled from any one source crate's own severity enum.
    pub severity: u8,
    /// Short display label.
    pub title: String,
}

/// One severity bucket's row in a dashboard summary widget.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeverityBucket {
    /// Numeric severity this bucket represents.
    pub severity: u8,
    /// Count of events at this severity.
    pub count: usize,
}

/// A single page of events, sorted highest-severity-first, ready for a
/// table widget to render.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventPage {
    /// Events on this page, in display order.
    pub items: Vec<SourceEvent>,
    /// 0-based page index returned.
    pub page: usize,
    /// Page size used.
    pub page_size: usize,
    /// Total events across all pages (before pagination).
    pub total: usize,
}

/// A dashboard summary widget's data: severity distribution plus which
/// source systems contributed events, ready to hand to a chart component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardSummary {
    /// Severity histogram, always present for severities 0..=4 (zero
    /// counts included, so the chart widget doesn't need to fill gaps).
    pub severity_buckets: Vec<SeverityBucket>,
    /// Distinct source system names present in the underlying event set,
    /// sorted alphabetically.
    pub sources: Vec<String>,
    /// Total event count.
    pub total_events: usize,
}
