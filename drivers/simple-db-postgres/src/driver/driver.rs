use std::sync::Arc;

use async_trait::async_trait;
use simple_db_core::{driver::{driver::DbDriver, executor::DbExecutor, transaction::DbTransaction}, query::{FindQuery, InsertQuery, PreparedDeleteQuery, PreparedFindQuery, PreparedInsertQuery, PreparedUpdateQuery, UpdateQuery}, types::{DbError, DbResult}};
use sqlx::{PgPool, postgres::PgPoolOptions};

use crate::{PostgresTransaction, driver::executor::PostgresExecutor, queries::{find::PostgresPreparedFindQuery, insert::PostgresPreparedInsertQuery, update::PostgresPreparedUpdateQuery}};

/// PostgreSQL database driver with connection pooling.
///
/// `PostgresDriver` manages a pool of PostgreSQL connections and implements the `DbDriver` trait
/// for query execution and transaction management.
///
/// # Example
/// ```ignore
/// let driver = PostgresDriver::connect("postgresql://user:pass@localhost/db").await?;
/// let cursor = driver.find(FindQuery::new("users")).await?;
/// ```
pub struct PostgresDriver {
    /// The underlying connection pool for executing queries.
    executor: PostgresExecutor,
}

impl PostgresDriver {
    /// Creates a new driver from an existing connection pool.
    pub fn new(pool: PgPool) -> Self {
        Self {
            executor: PostgresExecutor::Pool(pool),
        }
    }

    /// Establishes a new connection pool to the PostgreSQL server at `url`.
    ///
    /// The connection pool is configured with up to 5 concurrent connections.
    pub async fn connect(url: &str) -> DbResult<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(url)
            .await
            .map_err(DbError::driver)?;
        Ok(Self::new(pool))
    }

    /// Executes raw SQL directly without parameter binding.
    ///
    /// # Warning
    /// This method should only be used for DDL or administrative queries.
    /// Use the query builder API for parameterized queries to avoid SQL injection.
    pub async fn execute_raw(&self, sql: &str) -> DbResult<()> {
        let query = sqlx::query(sql);
        self.executor.execute(query)
            .await
            .map_err(DbError::driver)?;
        Ok(())
    }
}

#[async_trait]
impl DbExecutor for PostgresDriver {
    fn prepare_find(&self, query: FindQuery) -> DbResult<Box<dyn PreparedFindQuery + '_>> {
        Ok(Box::new(PostgresPreparedFindQuery::new(&self.executor, query)))
    }

    fn prepare_insert(&self, query: InsertQuery) -> DbResult<Box<dyn PreparedInsertQuery + '_>> {
        Ok(Box::new(PostgresPreparedInsertQuery::new(&self.executor, query)))
    }

    fn prepare_update(&self, query: UpdateQuery) -> DbResult<Box<dyn PreparedUpdateQuery + '_>> {
        Ok(Box::new(PostgresPreparedUpdateQuery::new(&self.executor, query)))
    }

    fn prepare_delete(&self, query: simple_db_core::query::DeleteQuery) -> DbResult<Box<dyn PreparedDeleteQuery + '_>> {
        Ok(Box::new(crate::queries::delete::PostgresPreparedDeleteQuery::new(&self.executor, query)))
    }
}

#[async_trait]
impl DbDriver for PostgresDriver {
    async fn begin(&self) -> DbResult<Arc<dyn DbTransaction>> {
        if let PostgresExecutor::Pool(pool) = &self.executor {
            let tx = pool.begin().await.map_err(DbError::driver)?;
            let pg_tx = PostgresTransaction::new(tx);
            Ok(Arc::new(pg_tx))
        } else {
            Err(DbError::Internal("Cannot start a transaction from an existing transaction".into()))
        }
    }

    async fn ping(&self) -> DbResult<()> {
        if let PostgresExecutor::Pool(pool) = &self.executor {
            pool.acquire().await.map_err(DbError::driver)?;
        }
        Ok(())
    }
}