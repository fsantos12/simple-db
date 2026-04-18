use simple_db_core::query::GroupDefinition;

/// Compiles a [`GroupDefinition`] into a comma-separated GROUP BY clause fragment.
///
/// Returns an empty string when there are no group fields.
pub fn compile_groups(groups: &GroupDefinition) -> String {
    if groups.is_empty() { return "".to_string() }

    let group_sql = groups.iter()
        .map(|col| col.to_string())
        .collect::<Vec<_>>()
        .join(", ");

    return group_sql;
}
