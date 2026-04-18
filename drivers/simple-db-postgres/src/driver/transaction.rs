use async_trait::async_trait;
use tokio::sync::Mutex;
use simple_db_core::{
    driver::{DbExecutor, DbTransaction},
    query::{DeleteQuery, FindQuery, InsertQuery, UpdateQuery},
    types::{DbCursor, DbError, DbResult},
};
use sqlx::Postgres;

use super::executor::{exec_delete, exec_find, exec_insert, exec_update};

#[derive(Debug, thiserror::Error)]
#[error("transaction has already been committed or rolled back")]
struct TransactionConsumedError;

/// A PostgreSQL transaction wrapping a sqlx connection held open for the transaction's lifetime.
///
/// Uses `Mutex<Option<...>>` so that:
/// - CRUD operations can borrow `&mut Transaction` while holding the lock.
/// - `commit` / `rollback` consume the transaction by calling `take()`.
///
/// # Example
///
/// ```rust,ignore
/// let tx = driver.begin().await?;
/// tx.insert(Query::insert("orders").insert(row)).await?;
/// tx.commit().await?;
/// ```
pub struct PostgresTransaction {
    tx: Mutex<Option<sqlx::Transaction<'static, Postgres>>>,
}

impl PostgresTransaction {
    /// Wraps an open sqlx transaction in a [`PostgresTransaction`].
    pub fn new(tx: sqlx::Transaction<'static, Postgres>) -> Self {
        Self { tx: Mutex::new(Some(tx)) }
    }
}

#[async_trait]
impl DbExecutor for PostgresTransaction {
    async fn find(&self, query: FindQuery) -> DbResult<Box<dyn DbCursor>> {
        let mut guard = self.tx.lock().await;
        let tx = guard.as_mut().ok_or_else(|| DbError::driver(TransactionConsumedError))?;
        exec_find(&mut **tx, query).await
    }

    async fn insert(&self, query: InsertQuery) -> DbResult<u64> {
        let mut guard = self.tx.lock().await;
        let tx = guard.as_mut().ok_or_else(|| DbError::driver(TransactionConsumedError))?;
        exec_insert(&mut **tx, query).await
    }

    async fn update(&self, query: UpdateQuery) -> DbResult<u64> {
        let mut guard = self.tx.lock().await;
        let tx = guard.as_mut().ok_or_else(|| DbError::driver(TransactionConsumedError))?;
        exec_update(&mut **tx, query).await
    }

    async fn delete(&self, query: DeleteQuery) -> DbResult<u64> {
        let mut guard = self.tx.lock().await;
        let tx = guard.as_mut().ok_or_else(|| DbError::driver(TransactionConsumedError))?;
        exec_delete(&mut **tx, query).await
    }
}

#[async_trait]
impl DbTransaction for PostgresTransaction {
    async fn commit(&self) -> DbResult<()> {
        let mut guard = self.tx.lock().await;
        let tx = guard.take().ok_or_else(|| DbError::driver(TransactionConsumedError))?;
        tx.commit().await.map_err(DbError::driver)
    }

    async fn rollback(&self) -> DbResult<()> {
        let mut guard = self.tx.lock().await;
        if let Some(tx) = guard.take() {
            tx.rollback().await.map_err(DbError::driver)?;
        }
        Ok(())
    }
}
