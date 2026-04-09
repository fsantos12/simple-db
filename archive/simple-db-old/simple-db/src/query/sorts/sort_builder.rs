//! Fluent builder API for sort ordering and null handling strategies.
//!
//! `SortBuilder` defines result ordering with support for ascending/descending
//! directions and null value placement control. Also supports random ordering.

use crate::query::sorts::sort::{Sort, SortDefinition};
use smol_str::SmolStr;

#[derive(Debug, Clone, Default)]
pub struct SortBuilder {
    items: SortDefinition,
}

impl SortBuilder {
    pub fn new() -> Self {
        Self { items: SortDefinition::new() }
    }

    /// Internal helper to push a sort instruction and return Self for chaining.
    fn add(mut self, sort: Sort) -> Self {
        self.items.push(sort);
        self
    }

    // --- Basic ---
    pub fn asc<F: Into<SmolStr>>(self, field: F) -> Self {
        self.add(Sort::Asc(field.into()))
    }

    pub fn desc<F: Into<SmolStr>>(self, field: F) -> Self {
        self.add(Sort::Desc(field.into()))
    }

    // --- Null Handling ---
    pub fn asc_nulls_first<F: Into<SmolStr>>(self, field: F) -> Self {
        self.add(Sort::AscNullsFirst(field.into()))
    }

    pub fn asc_nulls_last<F: Into<SmolStr>>(self, field: F) -> Self {
        self.add(Sort::AscNullsLast(field.into()))
    }

    pub fn desc_nulls_first<F: Into<SmolStr>>(self, field: F) -> Self {
        self.add(Sort::DescNullsFirst(field.into()))
    }

    pub fn desc_nulls_last<F: Into<SmolStr>>(self, field: F) -> Self {
        self.add(Sort::DescNullsLast(field.into()))
    }

    // --- Special Cases ---
    pub fn random(self) -> Self {
        self.add(Sort::Random)
    }

    // --- Finalization ---
    pub fn build(self) -> SortDefinition {
        self.items
    }
}