use std::sync::Arc;

use async_trait::async_trait;
use simple_db_core::{driver::{driver::DbDriver, executor::DbExecutor, transaction::DbTransaction}, query::{FindQuery, InsertQuery, PreparedDeleteQuery, PreparedFindQuery, PreparedInsertQuery, PreparedUpdateQuery, UpdateQuery}, types::{DbError, DbResult}};
use sqlx::{MySqlPool, mysql::MySqlPoolOptions};

use crate::{MySqlTransaction, driver::executor::MySqlExecutor, queries::{find::MySqlPreparedFindQuery, insert::MySqlPreparedInsertQuery, update::MySqlPreparedUpdateQuery}};

/// MySQL database driver with connection pooling.
///
/// `MySqlDriver` manages a pool of MySQL connections and implements the `DbDriver` trait
/// for query execution and transaction management.
///
/// # Example
/// ```ignore
/// let driver = MySqlDriver::connect("mysql://user:pass@localhost/db").await?;
/// let cursor = driver.find(FindQuery::new("users")).await?;
/// ```
pub struct MySqlDriver {
    /// The underlying connection pool for executing queries.
    executor: MySqlExecutor 
}

impl MySqlDriver {
    /// Creates a new driver from an existing connection pool.
    pub fn new(pool: MySqlPool) -> Self {
        Self { 
            executor: MySqlExecutor::Pool(pool) 
        }
    }

    /// Establishes a new connection pool to the MySQL server at `url`.
    ///
    /// The connection pool is configured with up to 5 concurrent connections.
    pub async fn connect(url: &str) -> DbResult<Self> {
        let pool = MySqlPoolOptions::new()
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
impl DbExecutor for MySqlDriver {
    fn prepare_find(&self, query: FindQuery) -> DbResult<Box<dyn PreparedFindQuery + '_>> {
        Ok(Box::new(MySqlPreparedFindQuery::new(&self.executor, query)))
    }

    fn prepare_insert(&self, query: InsertQuery) -> DbResult<Box<dyn PreparedInsertQuery + '_>> {
        Ok(Box::new(MySqlPreparedInsertQuery::new(&self.executor, query)))
    }

    fn prepare_update(&self, query: UpdateQuery) -> DbResult<Box<dyn PreparedUpdateQuery + '_>> {
        Ok(Box::new(MySqlPreparedUpdateQuery::new(&self.executor, query)))
    }

    fn prepare_delete(&self, query: simple_db_core::query::DeleteQuery) -> DbResult<Box<dyn PreparedDeleteQuery + '_>> {
        Ok(Box::new(crate::queries::delete::MySqlPreparedDeleteQuery::new(&self.executor, query)))
    }
}

#[async_trait]
impl DbDriver for MySqlDriver {
    async fn begin(&self) -> DbResult<Arc<dyn DbTransaction>> {
        if let MySqlExecutor::Pool(pool) = &self.executor {
            let tx = pool.begin().await.map_err(DbError::driver)?;
            let mysql_tx = MySqlTransaction::new(tx);
            Ok(Arc::new(mysql_tx))
        } else {
            Err(DbError::Internal("Cannot start a transaction from an existing transaction".into()))
        }
    }

    async fn ping(&self) -> DbResult<()> {
        if let MySqlExecutor::Pool(pool) = &self.executor {
            pool.acquire().await.map_err(DbError::driver)?;
        }
        Ok(())
    }
}