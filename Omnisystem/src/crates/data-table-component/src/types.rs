//! Table data model.

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// A single table cell value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Cell {
    /// Free text.
    Text(String),
    /// A numeric value.
    Number(f64),
    /// A boolean flag.
    Bool(bool),
    /// An absent value.
    Null,
}

impl Cell {
    /// Compare two cells for ordering purposes. Cross-variant comparisons
    /// fall back to comparing their type-independent text form so a sort
    /// always produces a total order, `Null` sorting last regardless of
    /// direction.
    pub fn cmp_for_sort(&self, other: &Cell) -> Ordering {
        match (self, other) {
            (Cell::Null, Cell::Null) => Ordering::Equal,
            (Cell::Null, _) => Ordering::Greater,
            (_, Cell::Null) => Ordering::Less,
            (Cell::Number(a), Cell::Number(b)) => a.partial_cmp(b).unwrap_or(Ordering::Equal),
            (Cell::Bool(a), Cell::Bool(b)) => a.cmp(b),
            (Cell::Text(a), Cell::Text(b)) => a.cmp(b),
            _ => self.display().cmp(&other.display()),
        }
    }

    /// Render the cell as display text, for cross-type comparison/filtering.
    pub fn display(&self) -> String {
        match self {
            Cell::Text(s) => s.clone(),
            Cell::Number(n) => n.to_string(),
            Cell::Bool(b) => b.to_string(),
            Cell::Null => String::new(),
        }
    }
}

/// A sort direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortDirection {
    /// Smallest/earliest first.
    Ascending,
    /// Largest/latest first.
    Descending,
}

/// A single row: one cell per column, in column order.
pub type Row = Vec<Cell>;

/// A tabular dataset: named columns plus rows of aligned cells.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    /// Column names, in display order.
    pub columns: Vec<String>,
    /// Rows of data; each row's cells align positionally with `columns`.
    pub rows: Vec<Row>,
}
