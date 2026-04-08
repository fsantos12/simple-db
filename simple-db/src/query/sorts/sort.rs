//! Sort instructions for result ordering and null value placement.
//!
//! The `Sort` enum defines result ordering with support for ascending/descending
//! directions, null placement control, and random ordering for sampling.

pub type SortDefinition = Vec<Sort>;

#[derive(Debug, Clone)]
pub enum Sort {
    // --- Basic ---
    Asc(Box<String>),
    Desc(Box<String>),

    // --- Null Handling ---
    AscNullsFirst(Box<String>),
    AscNullsLast(Box<String>),
    DescNullsFirst(Box<String>),
    DescNullsLast(Box<String>),

    // --- Special Cases ---
    Random,
}