use crate::query::{FilterBuilder, FilterDefinition};

/// DELETE query builder for removing data from a collection.
///
/// Supports:
/// - WHERE filter conditions to target specific rows
/// - Multiple filter conditions (implicit AND logic)
///
/// # Example
///
/// ```rust,ignore
/// let query = Query::delete("users")
///     .filter(filter!(eq("id", 42)));
/// ```
///
/// # Safety Note
///
/// If no filters are specified, the delete applies to ALL rows in the collection.
/// Always use `.filter()` unless you explicitly want to delete everything.
#[derive(Debug, Clone)]
pub struct DeleteQuery {
    pub collection: String,
    pub filters: FilterDefinition,
}

impl DeleteQuery {
    /// Creates a new delete query for the given collection.
    pub fn new(collection: impl Into<String>) -> Self {
        Self {
            collection: collection.into(),
            filters: FilterDefinition::new(),
        }
    }

    /// Adds filter conditions (WHERE clause) from a pre-built definition.
    ///
    /// Multiple calls use implicit AND logic.
    pub fn filter(mut self, filters: FilterDefinition) -> Self {
        self.filters.extend(filters);
        self
    }

    /// Adds filter conditions via a builder closure.
    pub fn with_filter_builder<F>(mut self, build: F) -> Self
    where
        F: FnOnce(FilterBuilder) -> FilterBuilder,
    {
        self.filters.extend(build(FilterBuilder::new()).build());
        self
    }
}
