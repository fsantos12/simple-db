use smallvec::SmallVec;
use smol_str::SmolStr;

use crate::builders::{SortDefinition, sorts::Sort};

/// Fluent builder for constructing sort ordering and null handling strategies.
///
/// `SortBuilder` defines result ordering with support for ascending/descending
/// directions and null value placement control. Also supports random ordering.
pub struct SortBuilder(SortDefinition);

impl SortBuilder {
    /// Create a new SortBuilder.
    pub fn new() -> Self {
        Self(SmallVec::new())
    }

    /// Add a sort instruction and return self for chaining.
    fn add(mut self, sort: Sort) -> Self {
        self.0.push(sort);
        self
    }

    /// Add multiple sort instructions at once.
    pub fn extend<I>(mut self, sorts: I) -> Self
    where I: IntoIterator<Item = Sort>,
    {
        self.0.extend(sorts);
        self
    }

    // --- Basic ---
    /// Sort ascending.
    pub fn asc<F: Into<SmolStr>>(self, field: F) -> Self {
        self.add(Sort::Asc(field.into()))
    }

    /// Sort descending.
    pub fn desc<F: Into<SmolStr>>(self, field: F) -> Self {
        self.add(Sort::Desc(field.into()))
    }

    // --- Null Handling ---
    /// Sort ascending with null values first.
    pub fn asc_nulls_first<F: Into<SmolStr>>(self, field: F) -> Self {
        self.add(Sort::AscNullsFirst(field.into()))
    }

    /// Sort ascending with null values last.
    pub fn asc_nulls_last<F: Into<SmolStr>>(self, field: F) -> Self {
        self.add(Sort::AscNullsLast(field.into()))
    }

    /// Sort descending with null values first.
    pub fn desc_nulls_first<F: Into<SmolStr>>(self, field: F) -> Self {
        self.add(Sort::DescNullsFirst(field.into()))
    }

    /// Sort descending with null values last.
    pub fn desc_nulls_last<F: Into<SmolStr>>(self, field: F) -> Self {
        self.add(Sort::DescNullsLast(field.into()))
    }

    // --- Special Cases ---
    /// Sort in random order.
    pub fn random(self) -> Self {
        self.add(Sort::Random)
    }

    /// Finalize the builder and return the sorts.
    pub fn build(self) -> SmallVec<[Sort; 4]> {
        self.0
    }
}

impl Default for SortBuilder {
    fn default() -> Self {
        Self::new()
    }
}
