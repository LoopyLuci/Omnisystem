//! Demo CLI: sort, filter, and paginate a small table.

use data_table_component::{Cell, SortDirection, Table};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let table = Table::new(
        vec!["name".into(), "score".into()],
        vec![
            vec![Cell::Text("Bea".into()), Cell::Number(88.0)],
            vec![Cell::Text("Al".into()), Cell::Number(95.0)],
            vec![Cell::Text("Cy".into()), Cell::Number(70.0)],
        ],
    );
    let sorted = table.sort_by("score", SortDirection::Descending)?;
    for row in table.paginate(0, 2)? {
        println!("page0 row: {row:?}");
    }
    println!("sorted by score desc: {:?}", sorted.rows.iter().map(|r| r[0].display()).collect::<Vec<_>>());
    Ok(())
}
