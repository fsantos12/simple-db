//! Find query builder for retrieving records from a collection.

use crate::builders::{
    filters::{FilterBuilder, FilterDefinition},
    groups::{GroupBuilder, GroupDefinition},
    projections::{ProjectionBuilder, ProjectionDefinition},
    sorts::{SortBuilder, SortDefinition},
};

/// Retrieve and filter records from a collection with optional projections, sorting, grouping, and pagination.
#[derive(Debug, Clone)]
pub struct FindQuery {
    pub collection: String,
    pub projections: ProjectionDefinition,
    pub filters: FilterDefinition,
    pub sorts: SortDefinition,
    pub groups: GroupDefinition,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl FindQuery {
    pub fn new<S: Into<String>>(collection: S) -> Self {
        Self {
            collection: collection.into(),
            projections: ProjectionDefinition::new(),
            filters: FilterDefinition::new(),
            sorts: SortDefinition::new(),
            groups: GroupDefinition::new(),
            limit: None,
            offset: None,
        }
    }

    /// Selects specific fields or aggregations using a closure.
    pub fn project<F>(mut self, build: F) -> Self
    where
        F: FnOnce(ProjectionBuilder) -> ProjectionBuilder,
    {
        let builder = build(ProjectionBuilder::new());
        self.projections.extend(builder.build());
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

    /// Defines the ordering of the result set.
    pub fn order_by<F>(mut self, build: F) -> Self
    where
        F: FnOnce(SortBuilder) -> SortBuilder,
    {
        let builder = build(SortBuilder::new());
        self.sorts.extend(builder.build());
        self
    }

    /// Defines groupings for aggregate queries.
    pub fn group_by<F>(mut self, build: F) -> Self
    where
        F: FnOnce(GroupBuilder) -> GroupBuilder,
    {
        let builder = build(GroupBuilder::new());
        self.groups.extend(builder.build());
        self
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }
}
