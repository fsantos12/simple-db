//! Sort and ordering definitions for result sets.
//!
//! Provides the `Sort` enum for result ordering specifications and `SortBuilder`
//! for fluent construction with ascending/descending directions and null placement control.

mod sort;
mod sort_builder;

pub use sort::SortDefinition;
pub use sort::Sort;
pub use sort_builder::SortBuilder;