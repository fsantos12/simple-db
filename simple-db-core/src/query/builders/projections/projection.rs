//! Field selection and aggregation definitions for result shaping.
//!
//! The `Projection` enum supports selecting individual fields, applying aggregate
//! functions (Count, Sum, Avg, Min, Max), and aliasing results for custom output.

use smol_str::SmolStr;

#[derive(Debug, Clone)]
pub enum Projection {
    /// A single column: `SELECT price`
    Field(SmolStr),

    /// A recursive alias: `inner_projection AS alias_name`.
    ///
    /// Allows complex expressions like `SUM(price) AS total`.
    Aliased(Box<Projection>, SmolStr),

    // --- Aggregations ---

    /// `COUNT(*)` — counts all rows, including those with NULL values.
    CountAll,

    /// `COUNT(field)` — counts non-null values of the given column.
    Count(SmolStr),

    /// `SUM(field)` — sum of all non-null values of the given column.
    Sum(SmolStr),

    /// `AVG(field)` — arithmetic average of non-null values.
    Avg(SmolStr),

    /// `MIN(field)` — smallest non-null value in the column.
    Min(SmolStr),

    /// `MAX(field)` — largest non-null value in the column.
    Max(SmolStr),
}

impl Projection {
    /// Wraps any projection with an alias: `projection AS alias`.
    pub fn r#as<S: Into<SmolStr>>(self, alias: S) -> Self {
        Projection::Aliased(Box::new(self), alias.into())
    }
}
