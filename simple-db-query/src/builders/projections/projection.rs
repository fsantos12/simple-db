//! Field selection and aggregation definitions for result shaping.
//!
//! The `Projection` enum supports selecting individual fields, applying aggregate
//! functions (Count, Sum, Avg, Min, Max), and aliasing results for custom output.

use smol_str::SmolStr;

#[derive(Debug, Clone)]
pub enum Projection {
    /// A single column: 'price'
    Field(SmolStr),

    /// A recursive alias: 'inner_projection AS alias_name'
    /// Allows complex structures like 'SUM(price) AS total'. 
    Aliased(Box<Projection>, SmolStr),

    // --- Aggregations ---
    CountAll,
    Count(SmolStr),
    Sum(SmolStr),
    Avg(SmolStr),
    Min(SmolStr),
    Max(SmolStr),
}

impl Projection {
    /// Helper to wrap any projection with an alias.
    pub fn r#as<S: Into<SmolStr>>(self, alias: S) -> Self {
        Projection::Aliased(Box::new(self), alias.into())
    }
}