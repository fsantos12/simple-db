use smallvec::SmallVec;
use smol_str::SmolStr;

use crate::builders::{ProjectionDefinition, projections::Projection};

/// Fluent builder for constructing field selection and aggregation projections.
///
/// `ProjectionBuilder` enables selecting specific fields and applying aggregate
/// functions (count, sum, avg, min, max) to shape query results. Projections
/// can include field aliases for renamed output columns using the `.as()` method.
pub struct ProjectionBuilder(ProjectionDefinition);

impl ProjectionBuilder {
    /// Create a new ProjectionBuilder.
    pub fn new() -> Self {
        Self(SmallVec::new())
    }

    /// Add a projection and return self for chaining.
    fn add(mut self, projection: Projection) -> Self {
        self.0.push(projection);
        self
    }

    /// Add multiple projections at once.
    pub fn extend<I>(mut self, projections: I) -> Self
    where I: IntoIterator<Item = Projection>,
    {
        self.0.extend(projections);
        self
    }

    // --- Basic Field Selection ---
    /// Select a single field.
    pub fn field<F: Into<SmolStr>>(self, field: F) -> Self {
        self.add(Projection::Field(field.into()))
    }

    // --- Aggregations ---
    /// Count all rows.
    pub fn count_all(self) -> Self {
        self.add(Projection::CountAll)
    }

    /// Count non-null values of a field.
    pub fn count<F: Into<SmolStr>>(self, field: F) -> Self {
        self.add(Projection::Count(field.into()))
    }

    /// Sum the values of a field.
    pub fn sum<F: Into<SmolStr>>(self, field: F) -> Self {
        self.add(Projection::Sum(field.into()))
    }

    /// Calculate the average of a field.
    pub fn avg<F: Into<SmolStr>>(self, field: F) -> Self {
        self.add(Projection::Avg(field.into()))
    }

    /// Find the minimum value of a field.
    pub fn min<F: Into<SmolStr>>(self, field: F) -> Self {
        self.add(Projection::Min(field.into()))
    }

    /// Find the maximum value of a field.
    pub fn max<F: Into<SmolStr>>(self, field: F) -> Self {
        self.add(Projection::Max(field.into()))
    }

    /// Finalize the builder and return the projections.
    pub fn build(self) -> SmallVec<[Projection; 10]> {
        self.0
    }
}

impl Default for ProjectionBuilder {
    fn default() -> Self {
        Self::new()
    }
}
