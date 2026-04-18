use simple_db_core::{query::InsertQuery, types::DbValue};

/// Returns the PostgreSQL positional placeholder for the given 1-based index: `$N`.
fn placeholder(position: usize) -> String {
    format!("${}", position)
}

/// Compiles an [`InsertQuery`] into a PostgreSQL INSERT statement and its bound parameters.
///
/// Generates a multi-row `INSERT INTO … VALUES ($1, $2), ($3, $4)` form when multiple
/// rows are provided. Returns an empty string if the query has no rows.
pub fn compile_insert_query(query: InsertQuery) -> (String, Vec<DbValue>) {
    if query.values.is_empty() { return (String::new(), vec![]); }

    let columns: Vec<String> = query.values[0].iter().map(|(col, _)| col.clone()).collect();
    let mut sql = String::with_capacity(128);

    sql.push_str("INSERT INTO ");
    sql.push_str(&query.collection);
    sql.push_str(" (");
    sql.push_str(&columns.join(", "));
    sql.push_str(") VALUES ");

    let total_rows = query.values.len();
    let columns_per_row = columns.len();

    let mut parameters = Vec::with_capacity(total_rows * columns_per_row);
    let mut row_placeholders = Vec::with_capacity(total_rows);
    let mut placeholder_index = 1;

    for row in query.values {
        let placeholders = (0..columns_per_row)
            .map(|_| { let ph = placeholder(placeholder_index); placeholder_index += 1; ph })
            .collect::<Vec<_>>()
            .join(", ");
        row_placeholders.push(format!("({})", placeholders));
        for (_, value) in row {
            parameters.push(value);
        }
    }

    sql.push_str(&row_placeholders.join(", "));
    (sql, parameters)
}
