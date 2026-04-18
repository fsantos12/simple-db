use simple_db_core::{query::DeleteQuery, types::DbValue};

use crate::builders::compile_filters;

/// Compiles a [`DeleteQuery`] into a PostgreSQL DELETE statement and its bound parameters.
///
/// If no filters are set, deletes all rows in the collection.
pub fn compile_delete_query(query: DeleteQuery) -> (String, Vec<DbValue>) {
    let (filter_sql, filter_params) = compile_filters(&query.filters);

    let exact_sql_capacity = 19 + query.collection.len() + filter_sql.len();
    let mut sql = String::with_capacity(exact_sql_capacity);

    sql.push_str("DELETE FROM ");
    sql.push_str(&query.collection);

    if !filter_sql.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&filter_sql);
    }

    (sql, filter_params)
}
