//! Fluent builder API for field selection and aggregation functions.
//!
//! `ProjectionBuilder` enables selecting specific fields and applying aggregate
//! functions (count, sum, avg, min, max) to shape query results. Projections
//! can include field aliases for renamed output columns.

use crate::query::projections::{Projection, ProjectionDefinition};
use smol_str::SmolStr;

#[derive(Debug, Clone, Default)]
pub struct ProjectionBuilder {
    items: ProjectionDefinition,
}

impl ProjectionBuilder {
    pub fn new() -> Self {
        Self { items: ProjectionDefinition::new() }
    }

    /// Internal helper to push a projection and return Self for chaining.
    fn add(mut self, projection: Projection) -> Self {
        self.items.push(projection);
        self
    }

    // --- Basic ---
    pub fn field<F: Into<SmolStr>>(self, field: F) -> Self {
        self.add(Projection::Field(field.into()))
    }

    pub fn field_as<F: Into<SmolStr>, A: Into<SmolStr>>(self, field: F, alias: A) -> Self {
        self.add(Projection::Field(field.into()).r#as(alias))
    }

    // --- Aggregations ---
    pub fn count_all(self) -> Self {
        self.add(Projection::CountAll)
    }

    pub fn count<F: Into<SmolStr>>(self, field: F) -> Self {
        self.add(Projection::Count(field.into()))
    }

    pub fn sum<F: Into<SmolStr>>(self, field: F) -> Self {
        self.add(Projection::Sum(field.into()))
    }

    pub fn avg<F: Into<SmolStr>>(self, field: F) -> Self {
        self.add(Projection::Avg(field.into()))
    }

    pub fn min<F: Into<SmolStr>>(self, field: F) -> Self {
        self.add(Projection::Min(field.into()))
    }

    pub fn max<F: Into<SmolStr>>(self, field: F) -> Self {
        self.add(Projection::Max(field.into()))
    }

    // --- Finalization ---
    pub fn build(self) -> ProjectionDefinition {
        self.items
    }
}