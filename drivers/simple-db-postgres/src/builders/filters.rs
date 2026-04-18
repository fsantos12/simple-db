use simple_db_core::{query::{Filter, FilterDefinition}, types::DbValue};

/// Compiles a [`FilterDefinition`] into a PostgreSQL `WHERE` clause fragment and its bound parameters.
///
/// Delegates to [`compile_filters_with_offset`] starting at parameter index `$1`.
pub fn compile_filters(filters: &FilterDefinition) -> (String, Vec<DbValue>) {
    compile_filters_with_offset(filters, 1)
}

/// Like [`compile_filters`] but starts numbering parameters from `start_index`.
///
/// This is used by [`compile_update_query`](crate::queries::update::compile_update_query)
/// to continue the `$N` sequence after the SET clause parameters.
pub fn compile_filters_with_offset(filters: &FilterDefinition, start_index: usize) -> (String, Vec<DbValue>) {
    if filters.is_empty() { return ("".to_string(), vec![]) }

    let mut sql_parts = Vec::new();
    let mut values = Vec::new();
    let mut parameter_index = start_index;

    for filter in filters {
        let (sql, mut params, next_index) = compile_filter(filter, parameter_index);
        sql_parts.push(sql);
        values.append(&mut params);
        parameter_index = next_index;
    }

    let final_sql = sql_parts.join(" AND ");
    (final_sql, values)
}

/// Joins a slice of filters with the given logical operator and wraps the result in parentheses.
/// Returns the next available parameter index so callers can continue numbering.
fn compile_logical_filters(filters: &[Filter], operator: &str, parameter_index: usize) -> (String, Vec<DbValue>, usize) {
    if filters.is_empty() { return ("".to_string(), vec![], parameter_index) }

    let mut sql_parts = Vec::new();
    let mut values = Vec::new();
    let mut current_index = parameter_index;

    for filter in filters {
        let (sql, mut params, next_index) = compile_filter(filter, current_index);
        sql_parts.push(sql);
        values.append(&mut params);
        current_index = next_index;
    }

    let final_sql = format!("({})", sql_parts.join(operator));
    (final_sql, values, current_index)
}

/// Compiles a single [`Filter`] variant into a SQL fragment, its bound parameters, and
/// the next available `$N` parameter index.
fn compile_filter(filter: &Filter, parameter_index: usize) -> (String, Vec<DbValue>, usize) {
    match filter {
        Filter::IsNull(smol_str) => (format!("{} IS NULL", smol_str), vec![], parameter_index),
        Filter::IsNotNull(smol_str) => (format!("{} IS NOT NULL", smol_str), vec![], parameter_index),

        Filter::Eq(smol_str, db_value) => (format!("{} = ${}", smol_str, parameter_index), vec![db_value.clone()], parameter_index + 1),
        Filter::Neq(smol_str, db_value) => (format!("{} != ${}", smol_str, parameter_index), vec![db_value.clone()], parameter_index + 1),
        Filter::Lt(smol_str, db_value) => (format!("{} < ${}", smol_str, parameter_index), vec![db_value.clone()], parameter_index + 1),
        Filter::Lte(smol_str, db_value) => (format!("{} <= ${}", smol_str, parameter_index), vec![db_value.clone()], parameter_index + 1),
        Filter::Gt(smol_str, db_value) => (format!("{} > ${}", smol_str, parameter_index), vec![db_value.clone()], parameter_index + 1),
        Filter::Gte(smol_str, db_value) => (format!("{} >= ${}", smol_str, parameter_index), vec![db_value.clone()], parameter_index + 1),

        Filter::StartsWith(col, val) => (format!("{} LIKE ${} || '%'", col, parameter_index), vec![val.clone()], parameter_index + 1),
        Filter::NotStartsWith(col, val) => (format!("{} NOT LIKE ${} || '%'", col, parameter_index), vec![val.clone()], parameter_index + 1),
        Filter::EndsWith(col, val) => (format!("{} LIKE '%' || ${}", col, parameter_index), vec![val.clone()], parameter_index + 1),
        Filter::NotEndsWith(col, val) => (format!("{} NOT LIKE '%' || ${}", col, parameter_index), vec![val.clone()], parameter_index + 1),
        Filter::Contains(col, val) => (format!("{} LIKE '%' || ${} || '%'", col, parameter_index), vec![val.clone()], parameter_index + 1),
        Filter::NotContains(col, val) => (format!("{} NOT LIKE '%' || ${} || '%'", col, parameter_index), vec![val.clone()], parameter_index + 1),

        Filter::Regex(smol_str, smol_str1) => (format!("{} ~ ${}", smol_str, parameter_index), vec![DbValue::from_string(smol_str1.clone())], parameter_index + 1),

        Filter::Between(smol_str, (low, high)) => (format!("{} BETWEEN ${} AND ${}", smol_str, parameter_index, parameter_index + 1), vec![low.clone(), high.clone()], parameter_index + 2),
        Filter::NotBetween(smol_str, (low, high)) => (format!("{} NOT BETWEEN ${} AND ${}", smol_str, parameter_index, parameter_index + 1), vec![low.clone(), high.clone()], parameter_index + 2),

        Filter::In(smol_str, vals) => {
            if vals.is_empty() {
                return ("1=0".to_string(), vec![], parameter_index);
            }
            let placeholders = (0..vals.len())
                .map(|i| format!("${}", parameter_index + i))
                .collect::<Vec<_>>()
                .join(", ");
            let sql = format!("{} IN ({})", smol_str, placeholders);
            (sql, vals.clone(), parameter_index + vals.len())
        }
        Filter::NotIn(smol_str, vals) => {
            if vals.is_empty() {
                return ("1=1".to_string(), vec![], parameter_index);
            }
            let placeholders = (0..vals.len())
                .map(|i| format!("${}", parameter_index + i))
                .collect::<Vec<_>>()
                .join(", ");
            let sql = format!("{} NOT IN ({})", smol_str, placeholders);
            (sql, vals.clone(), parameter_index + vals.len())
        }

        Filter::And(filters) => compile_logical_filters(filters, " AND ", parameter_index),
        Filter::Or(filters) => compile_logical_filters(filters, " OR ", parameter_index),
        Filter::Not(filter) => {
            let (sql, params, next_index) = compile_filter(filter, parameter_index);
            (format!("NOT ({})", sql), params, next_index)
        }
    }
}
