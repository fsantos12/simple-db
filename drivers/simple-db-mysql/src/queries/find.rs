use simple_db_core::{query::FindQuery, types::DbValue};

use crate::builders::{compile_filters, compile_groups, compile_projections, compile_sorts};

/// Compiles a [`FindQuery`] into a MySQL SELECT statement and its bound parameters.
///
/// Handles SELECT, FROM, WHERE, GROUP BY, ORDER BY, LIMIT, and OFFSET.
/// Parameters use `?` positional placeholders.
pub fn compile_find_query(query: FindQuery) -> (String, Vec<DbValue>) {
    let (filter_sql, parameters) = compile_filters(&query.filters);
    let proj_sql = compile_projections(&query.projections);
    let group_sql = compile_groups(&query.groups);
    let sort_sql = compile_sorts(&query.sorts);

    let capacity = 64 + query.collection.len() + proj_sql.len() + filter_sql.len() + group_sql.len() + sort_sql.len();
    let mut sql = String::with_capacity(capacity);

    sql.push_str("SELECT ");
    if proj_sql.is_empty() {
        sql.push('*');
    } else {
        sql.push_str(&proj_sql);
    }

    sql.push_str(" FROM ");
    sql.push_str(&query.collection);

    if !filter_sql.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&filter_sql);
    }

    if !group_sql.is_empty() {
        sql.push_str(" GROUP BY ");
        sql.push_str(&group_sql);
    }

    if !sort_sql.is_empty() {
        sql.push_str(" ORDER BY ");
        sql.push_str(&sort_sql);
    }

    if let Some(limit) = query.limit {
        sql.push_str(&format!(" LIMIT {}", limit));
    }
    
    if let Some(offset) = query.offset {
        sql.push_str(&format!(" OFFSET {}", offset));
    }

    (sql, parameters)
}
