use simple_db_core::{query::UpdateQuery, types::DbValue};

use crate::builders::compile_filters;

/// Compiles an [`UpdateQuery`] into a MySQL UPDATE statement and its bound parameters.
///
/// Returns an empty string if there are no field updates. Filter parameters are appended
/// after the SET clause parameters.
pub fn compile_update_query(query: UpdateQuery) -> (String, Vec<DbValue>) {
    if query.updates.is_empty() { return (String::new(), vec![]); }

    let (filter_sql, mut filter_params) = compile_filters(&query.filters);

    let mut sql = String::with_capacity(128);
    let mut parameters = Vec::with_capacity(query.updates.len() + filter_params.len());

    sql.push_str("UPDATE ");
    sql.push_str(&query.collection);
    sql.push_str(" SET ");

    let mut set_clauses = Vec::with_capacity(query.updates.len());
    for (field, value) in query.updates {
        set_clauses.push(format!("{} = ?", field));
        parameters.push(value);
    }
    sql.push_str(&set_clauses.join(", "));

    if !filter_sql.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&filter_sql);
        parameters.append(&mut filter_params);
    }

    (sql, parameters)
}
