use std::sync::Arc;

use async_trait::async_trait;
use simple_db_core::{
    driver::{DbDriver, DbExecutor, DbTransaction},
    query::{DeleteQuery, FindQuery, InsertQuery, UpdateQuery},
    types::{DbCursor, DbError, DbResult},
};
use sqlx::SqlitePool;

use super::{
    executor::{exec_delete, exec_find, exec_insert, exec_update},
    SqliteTransaction,
};

/// Pool-backed SQLite driver.
///
/// Wraps a [`SqlitePool`] and implements [`DbDriver`] so it can be injected
/// into a [`DbContext`](simple_db_core::context::DbContext).
///
/// # Example
///
/// ```rust,ignore
/// let pool = SqlitePoolOptions::new().connect("sqlite://:memory:").await?;
/// let driver = SqliteDriver::new(pool);
/// let ctx = DbContext::new(Arc::new(driver));
/// ```
pub struct SqliteDriver {
    pub pool: SqlitePool,
}

impl SqliteDriver {
    /// Creates a new [`SqliteDriver`] wrapping the given connection pool.
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

// ==========================================
// 1. Implement the generic execution trait
// ==========================================
#[async_trait]
impl DbExecutor for SqliteDriver {
    async fn find(&self, query: FindQuery) -> DbResult<Box<dyn DbCursor>> {
        exec_find(&self.pool, query).await
    }

    async fn insert(&self, query: InsertQuery) -> DbResult<u64> {
        exec_insert(&self.pool, query).await
    }

    async fn update(&self, query: UpdateQuery) -> DbResult<u64> {
        exec_update(&self.pool, query).await
    }

    async fn delete(&self, query: DeleteQuery) -> DbResult<u64> {
        exec_delete(&self.pool, query).await
    }
}

// ==========================================
// 2. Implement the Driver-specific trait
// ==========================================
#[async_trait]
impl DbDriver for SqliteDriver {
    async fn begin(&self) -> DbResult<Arc<dyn DbTransaction>> {
        let tx = self.pool.begin().await.map_err(DbError::driver)?;
        Ok(Arc::new(SqliteTransaction::new(tx)))
    }

    async fn ping(&self) -> DbResult<()> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map_err(DbError::driver)?;
        Ok(())
    }
}
