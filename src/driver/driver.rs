use std::collections::HashMap;

use async_trait::async_trait;

use crate::{error::DbError, query::{DeleteQuery, FindQuery, InsertQuery, UpdateQuery}, value::DbValue};

/// A single row representation: a map where keys are field names and values are DbValues.
pub type DbRow = HashMap<String, DbValue>;

/// A specialized Result type for database operations.
pub type DbResult<T> = Result<T, DbError>;

/// The core interface that all database backends must implement.
#[async_trait]
pub trait Driver: Send + Sync {
    async fn find(&self, query: FindQuery) -> DbResult<Vec<DbRow>>;
    async fn insert(&self, query: InsertQuery) -> DbResult<u64>;
    async fn update(&self, query: UpdateQuery) -> DbResult<u64>;
    async fn delete(&self, query: DeleteQuery) -> DbResult<u64>;
}
