use std::sync::Arc;

use async_trait::async_trait;
use simple_db_core::{
    driver::{DbDriver, DbExecutor, DbTransaction},
    query::{DeleteQuery, FindQuery, InsertQuery, UpdateQuery},
    types::{DbCursor, DbError, DbResult},
};
use sqlx::PgPool;

use super::{executor::{exec_delete, exec_find, exec_insert, exec_update}, PostgresTransaction};

/// Pool-backed PostgreSQL driver.
///
/// Wraps a [`PgPool`] and implements [`DbDriver`] so it can be injected
/// into a [`DbContext`](simple_db_core::context::DbContext).
///
/// # Example
///
/// ```rust,ignore
/// let pool = PgPoolOptions::new().connect("postgres://localhost/mydb").await?;
/// let driver = PostgresDriver::new(pool);
/// let ctx = DbContext::new(Arc::new(driver));
/// ```
pub struct PostgresDriver {
    pub pool: PgPool,
}

impl PostgresDriver {
    /// Creates a new [`PostgresDriver`] wrapping the given connection pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DbExecutor for PostgresDriver {
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

#[async_trait]
impl DbDriver for PostgresDriver {
    async fn begin(&self) -> DbResult<Arc<dyn DbTransaction>> {
        let tx = self.pool.begin().await.map_err(DbError::driver)?;
        Ok(Arc::new(PostgresTransaction::new(tx)))
    }

    async fn ping(&self) -> DbResult<()> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map_err(DbError::driver)?;
        Ok(())
    }
}
