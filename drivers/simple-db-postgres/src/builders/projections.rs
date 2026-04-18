use simple_db_core::query::{Projection, ProjectionDefinition};

/// Compiles a [`ProjectionDefinition`] into a comma-separated SELECT column list.
///
/// Returns `"*"` when the projection list is empty (select all columns).
pub fn compile_projections(projections: &ProjectionDefinition) -> String {
    if projections.is_empty() { return "*".to_string() }

    let projection_sql = projections.iter()
        .map(|proj| compile_projection(proj))
        .collect::<Vec<_>>()
        .join(", ");

    return projection_sql;
}

/// Compiles a single [`Projection`] variant into its SQL representation.
fn compile_projection(projection: &Projection) -> String {
    match projection {
        Projection::Field(smol_str) => smol_str.to_string(),
        Projection::Aliased(projection, smol_str) => format!("{} AS {}", compile_projection(projection), smol_str),
        Projection::CountAll => "COUNT(*)".to_string(),
        Projection::Count(smol_str) => format!("COUNT({})", smol_str),
        Projection::Sum(smol_str) => format!("SUM({})", smol_str),
        Projection::Avg(smol_str) => format!("AVG({})", smol_str),
        Projection::Min(smol_str) => format!("MIN({})", smol_str),
        Projection::Max(smol_str) => format!("MAX({})", smol_str),
    }
}
