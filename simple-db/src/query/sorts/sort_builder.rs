//! Fluent builder API for sort ordering and null handling strategies.
//!
//! `SortBuilder` defines result ordering with support for ascending/descending
//! directions and null value placement control. Also supports random ordering.

use crate::query::sorts::sort::{Sort, SortDefinition};

#[derive(Debug, Clone, Default)]
pub struct SortBuilder {
    items: SortDefinition,
}

impl SortBuilder {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Internal helper to push a sort instruction and return Self for chaining.
    fn add(mut self, sort: Sort) -> Self {
        self.items.push(sort);
        self
    }

    // --- Basic ---
    pub fn asc<F: Into<String>>(self, field: F) -> Self {
        self.add(Sort::Asc(Box::new(field.into())))
    }

    pub fn desc<F: Into<String>>(self, field: F) -> Self {
        self.add(Sort::Desc(Box::new(field.into())))
    }

    // --- Null Handling ---
    pub fn asc_nulls_first<F: Into<String>>(self, field: F) -> Self {
        self.add(Sort::AscNullsFirst(Box::new(field.into())))
    }

    pub fn asc_nulls_last<F: Into<String>>(self, field: F) -> Self {
        self.add(Sort::AscNullsLast(Box::new(field.into())))
    }

    pub fn desc_nulls_first<F: Into<String>>(self, field: F) -> Self {
        self.add(Sort::DescNullsFirst(Box::new(field.into())))
    }

    pub fn desc_nulls_last<F: Into<String>>(self, field: F) -> Self {
        self.add(Sort::DescNullsLast(Box::new(field.into())))
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