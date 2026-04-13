//! Update query builder for modifying records in a collection.

use crate::{
    builders::filters::{FilterBuilder, FilterDefinition},
    query::insert::DataRow,
    types::DbValue,
};

/// Modify records in a collection matching specified filter conditions.
#[derive(Debug, Clone)]
pub struct UpdateQuery {
    pub collection: String,
    pub updates: DataRow,
    pub filters: FilterDefinition,
}

impl UpdateQuery {
    pub fn new<S: Into<String>>(collection: S) -> Self {
        Self {
            collection: collection.into(),
            updates: DataRow::new(),
            filters: FilterDefinition::new(),
        }
    }

    /// Merges a pre-built `DataRow` into the update set.
    pub fn set_row(mut self, row: DataRow) -> Self {
        self.updates.extend(row);
        self
    }

    /// Sets a single field to the given value.
    pub fn set<F: Into<String>, V: Into<DbValue>>(mut self, field: F, value: V) -> Self {
        self.updates.insert(field.into(), value.into());
        self
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
