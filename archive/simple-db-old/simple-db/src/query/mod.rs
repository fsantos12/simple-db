//! Query builders for database operations.
//!
//! This module provides builder APIs for constructing type-safe queries (Find, Insert,
//! Update, Delete). Each query type supports fluent composition with specialized
//! builders for filters, projections, sorts, and groups.

use crate::{query::{
    filters::{FilterBuilder, FilterDefinition}, groups::{GroupBuilder, GroupDefinition}, projections::{ProjectionBuilder, ProjectionDefinition}, sorts::{SortBuilder, SortDefinition}
}, types::{DbRow, DbValue}};

pub mod projections;
pub mod filters;
pub mod sorts;
pub mod groups;

// ==========================================
// Find
// ==========================================
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
    where F: FnOnce(ProjectionBuilder) -> ProjectionBuilder {
        let builder = build(ProjectionBuilder::new());
        self.projections.extend(builder.build());
        self
    }

    /// Adds filter conditions. Calling this multiple times appends with implicit AND logic.
    pub fn filter<F>(mut self, build: F) -> Self 
    where F: FnOnce(FilterBuilder) -> FilterBuilder {
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
    where F: FnOnce(SortBuilder) -> SortBuilder {
        let builder = build(SortBuilder::new());
        self.sorts.extend(builder.build());
        self
    }

    /// Defines groupings for aggregate queries.
    pub fn group_by<F>(mut self, build: F) -> Self 
    where F: FnOnce(GroupBuilder) -> GroupBuilder {
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

// ==========================================
// Insert
// ==========================================
/// Add one or more new records to a collection.
#[derive(Debug, Clone)]
pub struct InsertQuery {
    pub collection: String,
    pub values: Vec<DbRow>,
}

impl InsertQuery {
    pub fn new<S: Into<String>>(collection: S) -> Self {
        Self {
            collection: collection.into(),
            values: Vec::new(),
        }
    }

    /// Inserts a single row from a collection of key-value pairs.
    pub fn insert<I, K, V>(mut self, row: I) -> Self
    where I: IntoIterator<Item = (K, V)>, K: Into<String>, V: Into<DbValue> {
        let db_row: DbRow = row.into_iter().map(|(k, v)| (k.into(), v.into())).collect();
        self.values.push(db_row);
        self
    }

    /// Batch inserts multiple rows efficiently.
    pub fn bulk_insert<I, R, K, V>(mut self, rows: I) -> Self
    where I: IntoIterator<Item = R>, R: IntoIterator<Item = (K, V)>, K: Into<String>, V: Into<DbValue> {
        for row in rows {
            let db_row: DbRow = row.into_iter().map(|(k, v)| (k.into(), v.into())).collect();
            self.values.push(db_row);
        }
        self
    }

    /// Directly inserts pre-built DbRow objects.
    pub fn values(mut self, rows: Vec<DbRow>) -> Self {
        self.values.extend(rows);
        self
    }
}

// ==========================================
// Update
// ==========================================
/// Modify records in a collection matching specified filter conditions.
#[derive(Debug, Clone)]
pub struct UpdateQuery {
    pub collection: String,
    pub updates: DbRow,
    pub filters: FilterDefinition,
}

impl UpdateQuery {
    pub fn new<S: Into<String>>(collection: S) -> Self {
        Self {
            collection: collection.into(),
            updates: DbRow::new(),
            filters: FilterDefinition::new(),
        }
    }

    pub fn set_row(mut self, row: DbRow) -> Self {
        self.updates.0.extend(row.0);
        self
    }

    pub fn set<F: Into<String>, V: Into<DbValue>>(mut self, field: F, value: V) -> Self {
        self.updates.insert(field, value);
        self
    }

    pub fn filter<F>(mut self, build: F) -> Self 
    where F: FnOnce(FilterBuilder) -> FilterBuilder {
        let builder = build(FilterBuilder::new());
        self.filters.extend(builder.build());
        self
    }

    pub fn with_filters(mut self, filters: FilterDefinition) -> Self {
        self.filters.extend(filters);
        self
    }
}

// ==========================================
// Delete
// ==========================================
/// Remove records from a collection matching specified filter conditions.
#[derive(Debug, Clone)]
pub struct DeleteQuery {
    pub collection: String,
    pub filters: FilterDefinition,
}

impl DeleteQuery {
    pub fn new(collection: impl Into<String>) -> Self {
        Self {
            collection: collection.into(),
            filters: FilterDefinition::new(),
        }
    }

    /// Adds filter conditions. Calling this multiple times appends with implicit AND logic.
    pub fn filter<F>(mut self, build: F) -> Self 
    where F: FnOnce(FilterBuilder) -> FilterBuilder {
        let builder = build(FilterBuilder::new());
        self.filters.extend(builder.build());
        self
    }

    pub fn with_filters(mut self, filters: FilterDefinition) -> Self {
        self.filters.extend(filters);
        self
    }
}

// ==========================================
// Query Builder Entry Point
// ==========================================
/// Entry point for constructing queries using the builder pattern.
pub struct Query;

impl Query {
    pub fn find<C: Into<String>>(collection: C) -> FindQuery {
        FindQuery::new(collection)
    }

    pub fn insert<C: Into<String>>(collection: C) -> InsertQuery {
        InsertQuery::new(collection)
    }

    pub fn update<C: Into<String>>(collection: C) -> UpdateQuery {
        UpdateQuery::new(collection)
    }

    pub fn delete<C: Into<String>>(collection: C) -> DeleteQuery {
        DeleteQuery::new(collection)
    }
}