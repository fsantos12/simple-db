//! Fluent builder API for field selection and aggregation functions.
//!
//! `ProjectionBuilder` enables selecting specific fields and applying aggregate
//! functions (count, sum, avg, min, max) to shape query results. Projections
//! can include field aliases for renamed output columns.

use crate::query::projections::{Projection, ProjectionDefinition};

#[derive(Debug, Clone, Default)]
pub struct ProjectionBuilder {
    items: ProjectionDefinition,
}

impl ProjectionBuilder {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Internal helper to push a projection and return Self for chaining.
    fn add(mut self, projection: Projection) -> Self {
        self.items.push(projection);
        self
    }

    // --- Basic ---
    pub fn field<F: Into<String>>(self, field: F) -> Self {
        self.add(Projection::Field(Box::new(field.into())))
    }

    pub fn field_as<F: Into<String>, A: Into<String>>(self, field: F, alias: A) -> Self {
        self.add(Projection::Field(Box::new(field.into())).r#as(alias))
    }

    // --- Aggregations ---
    pub fn count_all(self) -> Self {
        self.add(Projection::CountAll)
    }

    pub fn count<F: Into<String>>(self, field: F) -> Self {
        self.add(Projection::Count(Box::new(field.into())))
    }

    pub fn sum<F: Into<String>>(self, field: F) -> Self {
        self.add(Projection::Sum(Box::new(field.into())))
    }

    pub fn avg<F: Into<String>>(self, field: F) -> Self {
        self.add(Projection::Avg(Box::new(field.into())))
    }

    pub fn min<F: Into<String>>(self, field: F) -> Self {
        self.add(Projection::Min(Box::new(field.into())))
    }

    pub fn max<F: Into<String>>(self, field: F) -> Self {
        self.add(Projection::Max(Box::new(field.into())))
    }

    // --- Finalization ---
    pub fn build(self) -> ProjectionDefinition {
        self.items
    }
}