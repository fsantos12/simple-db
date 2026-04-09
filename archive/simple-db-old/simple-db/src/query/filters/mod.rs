//! Filter condition builders and AST nodes.
//!
//! Provides the `Filter` enum for constructing query predicates and `FilterBuilder`
//! for fluent construction of filter chains with support for null checks, comparisons,
//! pattern matching, ranges, set membership, and logical operations.

mod filter;
mod filter_builder;

pub use filter::FilterDefinition;
pub use filter::Filter;
pub use filter_builder::FilterBuilder;