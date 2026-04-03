use async_trait::async_trait;

use crate::{types::{DbError, DbRow}, query::{DeleteQuery, FindQuery, InsertQuery, UpdateQuery}};

/// The core interface that all database backends must implement.
#[async_trait]
pub trait Driver: Send + Sync {
    async fn find(&self, query: FindQuery) -> Result<Vec<DbRow>, DbError>;
    async fn insert(&self, query: InsertQuery) -> Result<u64, DbError>;
    async fn update(&self, query: UpdateQuery) -> Result<u64, DbError>;
    async fn delete(&self, query: DeleteQuery) -> Result<u64, DbError>;
}
