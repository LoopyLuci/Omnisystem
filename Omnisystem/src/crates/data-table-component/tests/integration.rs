//! End-to-end: filter, sort, then paginate a table, checking the pipeline's
//! combined result is consistent.

use data_table_component::{Cell, SortDirection, Table};

#[test]
fn filter_sort_paginate_pipeline() {
    let table = Table::new(
        vec!["name".into(), "active".into(), "score".into()],
        vec![
            vec![Cell::Text("Ana".into()), Cell::Bool(true), Cell::Number(72.0)],
            vec![Cell::Text("Bo".into()), Cell::Bool(false), Cell::Number(91.0)],
            vec![Cell::Text("Cid".into()), Cell::Bool(true), Cell::Number(85.0)],
            vec![Cell::Text("Dee".into()), Cell::Bool(true), Cell::Number(60.0)],
        ],
    );

    let active_only = table.filter_by("active", |c| matches!(c, Cell::Bool(true))).unwrap();
    assert_eq!(active_only.rows.len(), 3);

    let sorted = active_only.sort_by("score", SortDirection::Descending).unwrap();
    let names: Vec<String> = sorted.rows.iter().map(|r| r[0].display()).collect();
    assert_eq!(names, vec!["Cid", "Ana", "Dee"]);

    let page = sorted.paginate(0, 2).unwrap();
    assert_eq!(page.len(), 2);
    assert_eq!(page[0][0].display(), "Cid");
    assert_eq!(sorted.page_count(2), 2);
}
