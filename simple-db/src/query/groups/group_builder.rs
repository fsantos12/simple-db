//! Fluent builder API for GROUP BY clause construction.
//!
//! `GroupBuilder` enables specifying one or more fields for grouping aggregate
//! queries and controlling result set partitioning.

use crate::query::groups::GroupDefinition;

/// A fluent builder for constructing a GROUP BY clause.
#[derive(Debug, Clone, Default)]
pub struct GroupBuilder {
    items: GroupDefinition,
}

impl GroupBuilder {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn field<F: Into<String>>(mut self, field: F) -> Self {
        self.items.push(Box::new(field.into()));
        self
    }

    pub fn fields<F, I>(mut self, fields: I) -> Self 
    where F: Into<String>, I: IntoIterator<Item = F> {
        for f in fields {
            self.items.push(Box::new(f.into()));
        }
        self
    }

    // --- Finalization ---
    pub fn build(self) -> GroupDefinition {
        self.items
    }
}