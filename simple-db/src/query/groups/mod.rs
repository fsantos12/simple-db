//! GROUP BY clause construction and aggregation grouping.
//!
//! Provides `GroupBuilder` for fluent construction of GROUP BY clauses used in
//! aggregate queries to partition results by one or more fields.

mod group_builder;

pub type GroupDefinition = Vec<Box<String>>;

pub use group_builder::GroupBuilder;