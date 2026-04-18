use simple_db_core::{query::{Filter, FilterDefinition}, types::DbValue};

/// Compiles a [`FilterDefinition`] into a MySQL `WHERE` clause fragment and its bound parameters.
///
/// Top-level filters are joined with `AND`. Returns an empty string when there are no filters.
pub fn compile_filters(filters: &FilterDefinition) -> (String, Vec<DbValue>) {
    if filters.is_empty() { return ("".to_string(), vec![]) }

    let mut sql_parts = Vec::new();
    let mut values = Vec::new();

    for filter in filters {
        let (sql, mut params) = compile_filter(filter);
        sql_parts.push(sql);
        values.append(&mut params);
    }

    let final_sql = sql_parts.join(" AND ");
    (final_sql, values)
}

/// Joins a slice of filters with the given logical operator and wraps the result in parentheses.
fn compile_logical_filters(filters: &[Filter], operator: &str) -> (String, Vec<DbValue>) {
    if filters.is_empty() { return ("".to_string(), vec![]) }

    let mut sql_parts = Vec::new();
    let mut values = Vec::new();

    for filter in filters {
        let (sql, mut params) = compile_filter(filter);
        sql_parts.push(sql);
        values.append(&mut params);
    }

    let final_sql = format!("({})", sql_parts.join(operator));
    (final_sql, values)
}

/// Compiles a single [`Filter`] variant into a SQL fragment and its bound parameters.
fn compile_filter(filter: &Filter) -> (String, Vec<DbValue>) {
    match filter {
        Filter::IsNull(smol_str) => (format!("{} IS NULL", smol_str), vec![]),
        Filter::IsNotNull(smol_str) => (format!("{} IS NOT NULL", smol_str), vec![]),

        Filter::Eq(smol_str, db_value) => (format!("{} = ?", smol_str), vec![db_value.clone()]),
        Filter::Neq(smol_str, db_value) => (format!("{} != ?", smol_str), vec![db_value.clone()]),
        Filter::Lt(smol_str, db_value) => (format!("{} < ?", smol_str), vec![db_value.clone()]),
        Filter::Lte(smol_str, db_value) => (format!("{} <= ?", smol_str), vec![db_value.clone()]),
        Filter::Gt(smol_str, db_value) => (format!("{} > ?", smol_str), vec![db_value.clone()]),
        Filter::Gte(smol_str, db_value) => (format!("{} >= ?", smol_str), vec![db_value.clone()]),

        Filter::StartsWith(col, val) => (format!("{} LIKE ? || '%'", col), vec![val.clone()]),
        Filter::NotStartsWith(col, val) => (format!("{} NOT LIKE ? || '%'", col), vec![val.clone()]),
        Filter::EndsWith(col, val) => (format!("{} LIKE '%' || ?", col), vec![val.clone()]),
        Filter::NotEndsWith(col, val) => (format!("{} NOT LIKE '%' || ?", col), vec![val.clone()]),
        Filter::Contains(col, val) => (format!("{} LIKE '%' || ? || '%'", col), vec![val.clone()]),
        Filter::NotContains(col, val) => (format!("{} NOT LIKE '%' || ? || '%'", col), vec![val.clone()]),

        Filter::Regex(smol_str, smol_str1) => (format!("{} REGEXP ?", smol_str), vec![DbValue::from_string(smol_str1.clone())]),

        Filter::Between(smol_str, (low, high)) => (format!("{} BETWEEN ? AND ?", smol_str), vec![low.clone(), high.clone()]),
        Filter::NotBetween(smol_str, (low, high)) => (format!("{} NOT BETWEEN ? AND ?", smol_str), vec![low.clone(), high.clone()]),

        Filter::In(smol_str, vals) => {
            if vals.is_empty() {
                return ("1=0".to_string(), vec![]);
            }
            let placeholders = vec!["?"; vals.len()].join(", ");
            let sql = format!("{} IN ({})", smol_str, placeholders);
            (sql, vals.clone())
        }
        Filter::NotIn(smol_str, vals) => {
            if vals.is_empty() {
                return ("1=1".to_string(), vec![]);
            }
            let placeholders = vec!["?"; vals.len()].join(", ");
            let sql = format!("{} NOT IN ({})", smol_str, placeholders);
            (sql, vals.clone())
        }

        Filter::And(filters) => compile_logical_filters(filters, " AND "),
        Filter::Or(filters) => compile_logical_filters(filters, " OR "),
        Filter::Not(filter) => {
            let (sql, params) = compile_filter(filter);
            (format!("NOT ({})", sql), params)
        }
    }
}
