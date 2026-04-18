use simple_db_core::{query::FindQuery, types::DbValue};

use crate::builders::{compile_filters, compile_groups, compile_projections, compile_sorts};

/// Compiles a [`FindQuery`] into a SQLite SELECT statement and its bound parameters.
///
/// Handles SELECT, FROM, WHERE, GROUP BY, ORDER BY, LIMIT, and OFFSET.
/// SQLite does not support OFFSET without LIMIT, so `-1` is used as an unlimited sentinel.
///
/// # Example
///
/// ```rust,ignore
/// let (sql, params) = compile_find_query(Query::find("users").filter(|b| b.eq("active", true)));
/// // sql = "SELECT * FROM users WHERE active = ?"
/// // params = [DbValue::from(true)]
/// ```
pub fn compile_find_query(query: FindQuery) -> (String, Vec<DbValue>) {
    let (filter_sql, parameters) = compile_filters(&query.filters);
    let proj_sql = compile_projections(&query.projections);
    let group_sql = compile_groups(&query.groups);
    let sort_sql = compile_sorts(&query.sorts);

    let capacity = 64 + query.collection.len() + proj_sql.len() + filter_sql.len() + group_sql.len() + sort_sql.len();
    let mut sql = String::with_capacity(capacity);

    // SELECT
    sql.push_str("SELECT ");
    if proj_sql.is_empty() {
        sql.push('*');
    } else {
        sql.push_str(&proj_sql);
    }

    // FROM
    sql.push_str(" FROM ");
    sql.push_str(&query.collection);

    // WHERE
    if !filter_sql.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&filter_sql);
    }

    // GROUP BY
    if !group_sql.is_empty() {
        sql.push_str(" GROUP BY ");
        sql.push_str(&group_sql);
    }

    // ORDER BY
    if !sort_sql.is_empty() {
        sql.push_str(" ORDER BY ");
        sql.push_str(&sort_sql);
    }

    // LIMIT / OFFSET — SQLite requires LIMIT before OFFSET
    match (query.limit, query.offset) {
        (Some(limit), Some(offset)) => {
            sql.push_str(&format!(" LIMIT {} OFFSET {}", limit, offset));
        }
        (Some(limit), None) => {
            sql.push_str(&format!(" LIMIT {}", limit));
        }
        (None, Some(offset)) => {
            // SQLite does not support OFFSET without LIMIT; use -1 for unlimited
            sql.push_str(&format!(" LIMIT -1 OFFSET {}", offset));
        }
        (None, None) => {}
    }

    (sql, parameters)
}