//! Projection and aggregation function definitions.
//!
//! Provides the `Projection` enum for field selection and aggregate functions
//! and `ProjectionBuilder` for fluent construction of SELECT clauses with aliasing.

mod projection;
mod projection_builder;

pub use projection::ProjectionDefinition;
pub use projection::Projection;
pub use projection_builder::ProjectionBuilder;