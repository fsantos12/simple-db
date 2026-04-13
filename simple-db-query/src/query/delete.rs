//! Delete query builder for removing records from a collection.

use crate::builders::filters::{FilterBuilder, FilterDefinition};

/// Remove records from a collection matching specified filter conditions.
#[derive(Debug, Clone)]
pub struct DeleteQuery {
    pub collection: String,
    pub filters: FilterDefinition,
}

impl DeleteQuery {
    pub fn new<S: Into<String>>(collection: S) -> Self {
        Self {
            collection: collection.into(),
            filters: FilterDefinition::new(),
        }
    }

    /// Adds filter conditions. Calling this multiple times appends with implicit AND logic.
    pub fn filter<F>(mut self, build: F) -> Self
    where
        F: FnOnce(FilterBuilder) -> FilterBuilder,
    {
        let builder = build(FilterBuilder::new());
        self.filters.extend(builder.build());
        self
    }

    pub fn with_filters(mut self, filters: FilterDefinition) -> Self {
        self.filters.extend(filters);
        self
    }
}
