use simple_db_core::{query::InsertQuery, types::DbValue};

/// Compiles an [`InsertQuery`] into a SQLite INSERT statement and its bound parameters.
///
/// Generates a multi-row `INSERT INTO … VALUES (?, ?), (?, ?)` form when multiple
/// rows are provided. Returns an empty string if the query has no rows.
///
/// # Example
///
/// ```rust,ignore
/// let (sql, params) = compile_insert_query(
///     Query::insert("users").insert([("name", "Alice"), ("age", 25i32)])
/// );
/// // sql = "INSERT INTO users (name, age) VALUES (?, ?)"
/// ```
pub fn compile_insert_query(query: InsertQuery) -> (String, Vec<DbValue>) {
    if query.values.is_empty() { return (String::new(), vec![]);}

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

    for row in query.values {
        row_placeholders.push(format!("({})", vec!["?"; columns_per_row].join(", ")));
        for (_, value) in row {
            parameters.push(value);
        }
    }

    sql.push_str(&row_placeholders.join(", "));
    (sql, parameters)
}