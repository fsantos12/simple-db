use std::sync::Arc;

use async_trait::async_trait;
use simple_db_core::{
    driver::{DbDriver, DbExecutor, DbTransaction},
    query::{DeleteQuery, FindQuery, InsertQuery, UpdateQuery},
    types::{DbCursor, DbError, DbResult},
};
use sqlx::MySqlPool;

use super::{executor::{exec_delete, exec_find, exec_insert, exec_update}, MysqlTransaction};

/// Pool-backed MySQL driver.
///
/// Wraps a [`MySqlPool`] and implements [`DbDriver`] so it can be injected
/// into a [`DbContext`](simple_db_core::context::DbContext).
///
/// # Example
///
/// ```rust,ignore
/// let pool = MySqlPoolOptions::new().connect("mysql://root:root@localhost/mydb").await?;
/// let driver = MysqlDriver::new(pool);
/// let ctx = DbContext::new(Arc::new(driver));
/// ```
pub struct MysqlDriver {
    pub pool: MySqlPool,
}

impl MysqlDriver {
    /// Creates a new [`MysqlDriver`] wrapping the given connection pool.
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DbExecutor for MysqlDriver {
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
impl DbDriver for MysqlDriver {
    async fn begin(&self) -> DbResult<Arc<dyn DbTransaction>> {
        let tx = self.pool.begin().await.map_err(DbError::driver)?;
        Ok(Arc::new(MysqlTransaction::new(tx)))
    }

    async fn ping(&self) -> DbResult<()> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map_err(DbError::driver)?;
        Ok(())
    }
}
