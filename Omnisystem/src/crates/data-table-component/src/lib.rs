//! In-memory data table operations: multi-column sort, predicate filtering,
//! and pagination. Deliberately UI-agnostic — this crate computes the row
//! set a table widget would render, not the widget itself.

#![warn(missing_docs)]

pub mod error;
pub mod types;

pub use error::{Error, Result};
pub use types::*;

use std::cmp::Ordering;

impl Table {
    /// Build a new table. Rows whose length doesn't match `columns` are
    /// still accepted (padded conceptually with `Null` at read time by
    /// treating missing cells as `Cell::Null` in [`Table::cell`]).
    pub fn new(columns: Vec<String>, rows: Vec<Row>) -> Self {
        Self { columns, rows }
    }

    fn column_index(&self, name: &str) -> Result<usize> {
        self.columns
            .iter()
            .position(|c| c == name)
            .ok_or_else(|| Error::UnknownColumn(name.to_string()))
    }

    fn cell<'a>(&self, row: &'a Row, idx: usize) -> &'a Cell {
        row.get(idx).unwrap_or(&Cell::Null)
    }

    /// Sort rows by a named column, returning a new table (stable sort —
    /// equal keys preserve their original relative order).
    pub fn sort_by(&self, column: &str, direction: SortDirection) -> Result<Table> {
        let idx = self.column_index(column)?;
        let mut rows = self.rows.clone();
        rows.sort_by(|a, b| Self::directional_cmp(self.cell(a, idx), self.cell(b, idx), direction));
        Ok(Table { columns: self.columns.clone(), rows })
    }

    /// Compare two cells honoring `direction`, while always sorting `Null`
    /// last regardless of direction (a `Null`/`Null` reversal would
    /// otherwise be a no-op, but reversing a `Null`/value comparison would
    /// wrongly move nulls to the front in descending order).
    fn directional_cmp(a: &Cell, b: &Cell, direction: SortDirection) -> Ordering {
        if matches!(a, Cell::Null) || matches!(b, Cell::Null) {
            return a.cmp_for_sort(b);
        }
        let ord = a.cmp_for_sort(b);
        match direction {
            SortDirection::Ascending => ord,
            SortDirection::Descending => ord.reverse(),
        }
    }

    /// Sort rows by multiple columns in priority order (first column is the
    /// primary key, later columns break ties).
    pub fn sort_by_columns(&self, keys: &[(&str, SortDirection)]) -> Result<Table> {
        let indexed: Vec<(usize, SortDirection)> = keys
            .iter()
            .map(|(name, dir)| self.column_index(name).map(|i| (i, *dir)))
            .collect::<Result<Vec<_>>>()?;
        let mut rows = self.rows.clone();
        rows.sort_by(|a, b| {
            for (idx, dir) in &indexed {
                let ord = Self::directional_cmp(self.cell(a, *idx), self.cell(b, *idx), *dir);
                if ord != Ordering::Equal {
                    return ord;
                }
            }
            Ordering::Equal
        });
        Ok(Table { columns: self.columns.clone(), rows })
    }

    /// Keep only rows where `predicate` returns true, given the row's cell
    /// value for `column`.
    pub fn filter_by(&self, column: &str, predicate: impl Fn(&Cell) -> bool) -> Result<Table> {
        let idx = self.column_index(column)?;
        let rows = self
            .rows
            .iter()
            .filter(|row| predicate(self.cell(row, idx)))
            .cloned()
            .collect();
        Ok(Table { columns: self.columns.clone(), rows })
    }

    /// Split rows into pages of `page_size` rows each and return page
    /// `page` (zero-indexed).
    pub fn paginate(&self, page: usize, page_size: usize) -> Result<Vec<Row>> {
        if page_size == 0 {
            return Err(Error::InvalidPageSize);
        }
        let total_pages = self.page_count(page_size);
        if total_pages > 0 && page >= total_pages {
            return Err(Error::PageOutOfRange(page, total_pages));
        }
        let start = page * page_size;
        let end = (start + page_size).min(self.rows.len());
        Ok(self.rows[start.min(self.rows.len())..end].to_vec())
    }

    /// Number of pages of `page_size` rows this table would produce.
    pub fn page_count(&self, page_size: usize) -> usize {
        if page_size == 0 || self.rows.is_empty() {
            return if self.rows.is_empty() { 0 } else { 1 };
        }
        self.rows.len().div_ceil(page_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Table {
        Table::new(
            vec!["name".into(), "age".into()],
            vec![
                vec![Cell::Text("Bea".into()), Cell::Number(30.0)],
                vec![Cell::Text("Al".into()), Cell::Number(45.0)],
                vec![Cell::Text("Cy".into()), Cell::Number(30.0)],
            ],
        )
    }

    #[test]
    fn sort_ascending_by_number() {
        let sorted = sample().sort_by("age", SortDirection::Ascending).unwrap();
        let ages: Vec<f64> = sorted.rows.iter().map(|r| match r[1] { Cell::Number(n) => n, _ => 0.0 }).collect();
        assert_eq!(ages, vec![30.0, 30.0, 45.0]);
    }

    #[test]
    fn sort_descending_by_text() {
        let sorted = sample().sort_by("name", SortDirection::Descending).unwrap();
        let names: Vec<String> = sorted.rows.iter().map(|r| r[0].display()).collect();
        assert_eq!(names, vec!["Cy", "Bea", "Al"]);
    }

    #[test]
    fn sort_is_stable_on_ties() {
        let sorted = sample().sort_by("age", SortDirection::Ascending).unwrap();
        // Bea and Cy both have age 30; Bea appeared first in the source.
        assert_eq!(sorted.rows[0][0].display(), "Bea");
        assert_eq!(sorted.rows[1][0].display(), "Cy");
    }

    #[test]
    fn sort_rejects_unknown_column() {
        assert_eq!(
            sample().sort_by("missing", SortDirection::Ascending).unwrap_err(),
            Error::UnknownColumn("missing".into())
        );
    }

    #[test]
    fn multi_column_sort_breaks_ties() {
        let sorted = sample()
            .sort_by_columns(&[("age", SortDirection::Ascending), ("name", SortDirection::Descending)])
            .unwrap();
        let names: Vec<String> = sorted.rows.iter().map(|r| r[0].display()).collect();
        assert_eq!(names, vec!["Cy", "Bea", "Al"]);
    }

    #[test]
    fn filter_keeps_matching_rows_only() {
        let filtered = sample()
            .filter_by("age", |c| matches!(c, Cell::Number(n) if *n >= 40.0))
            .unwrap();
        assert_eq!(filtered.rows.len(), 1);
        assert_eq!(filtered.rows[0][0].display(), "Al");
    }

    #[test]
    fn filter_rejects_unknown_column() {
        assert!(sample().filter_by("nope", |_| true).is_err());
    }

    #[test]
    fn paginate_splits_rows_correctly() {
        let t = sample();
        let page0 = t.paginate(0, 2).unwrap();
        let page1 = t.paginate(1, 2).unwrap();
        assert_eq!(page0.len(), 2);
        assert_eq!(page1.len(), 1);
    }

    #[test]
    fn paginate_rejects_zero_page_size() {
        assert_eq!(sample().paginate(0, 0).unwrap_err(), Error::InvalidPageSize);
    }

    #[test]
    fn paginate_rejects_out_of_range_page() {
        assert!(matches!(sample().paginate(5, 2), Err(Error::PageOutOfRange(5, 2))));
    }

    #[test]
    fn page_count_matches_expected() {
        assert_eq!(sample().page_count(2), 2);
        assert_eq!(sample().page_count(10), 1);
    }

    #[test]
    fn null_cells_sort_last_regardless_of_direction() {
        let t = Table::new(
            vec!["v".into()],
            vec![vec![Cell::Number(1.0)], vec![Cell::Null], vec![Cell::Number(2.0)]],
        );
        let asc = t.sort_by("v", SortDirection::Ascending).unwrap();
        let desc = t.sort_by("v", SortDirection::Descending).unwrap();
        assert_eq!(asc.rows.last().unwrap()[0], Cell::Null);
        assert_eq!(desc.rows.last().unwrap()[0], Cell::Null);
    }
}
