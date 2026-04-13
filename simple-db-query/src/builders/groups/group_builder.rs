use smallvec::SmallVec;
use smol_str::SmolStr;

use crate::builders::GroupDefinition;

/// Fluent builder for constructing GROUP BY clauses.
///
/// `GroupBuilder` enables specifying one or more fields for grouping aggregate
/// queries and controlling result set partitioning.
pub struct GroupBuilder(GroupDefinition);

impl GroupBuilder {
    /// Create a new GroupBuilder.
    pub fn new() -> Self {
        Self(SmallVec::new())
    }

    /// Add a single field to group by.
    pub fn field<F: Into<SmolStr>>(mut self, field: F) -> Self {
        self.0.push(field.into());
        self
    }

    /// Add multiple fields to group by.
    pub fn fields<F, I>(mut self, fields: I) -> Self 
    where F: Into<SmolStr>, I: IntoIterator<Item = F> {
        for f in fields {
            self.0.push(f.into());
        }
        self
    }

    /// Finalize the builder and return the group definitions.
    pub fn build(self) -> SmallVec<[SmolStr; 4]> {
        self.0
    }
}

impl Default for GroupBuilder {
    fn default() -> Self {
        Self::new()
    }
}
