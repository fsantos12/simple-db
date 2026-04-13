//! Query builders for database operations.
//!
//! This module provides builder APIs for constructing type-safe queries (Find, Insert,
//! Update, Delete). Each query type supports fluent composition with specialized
//! builders for filters, projections, sorts, and groups.

pub mod delete;
pub mod find;
pub mod insert;
pub mod update;

pub use delete::DeleteQuery;
pub use find::FindQuery;
pub use insert::{DataRow, InsertQuery};
pub use update::UpdateQuery;

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
