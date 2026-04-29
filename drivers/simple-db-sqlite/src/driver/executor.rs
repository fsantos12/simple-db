use std::sync::Arc;
use tokio::sync::Mutex;
use sqlx::{Sqlite, SqlitePool, Transaction, sqlite::{SqliteArguments, SqliteQueryResult}, query::Query};

/// Executes sqlx queries against either a connection pool or an active transaction.
///
/// Internally wraps either a `SqlitePool` for general query execution or an
/// `Arc<Mutex<Option<Transaction<Sqlite>>>>` for executing within a transaction.
/// This enum allows query builders to remain agnostic about the execution context.
pub(crate) enum SqliteExecutor {
    /// Executes queries against the connection pool.
    Pool(SqlitePool),
    /// Executes queries against an active transaction.
    Transaction(Arc<Mutex<Option<Transaction<'static, Sqlite>>>>),
}

impl SqliteExecutor {
    /// Executes a query and returns the result (affected row count, etc.).
    pub(crate) async fn execute<'q>(&self, query: Query<'q, Sqlite, SqliteArguments<'q>>) -> sqlx::Result<SqliteQueryResult> {
        match self {
            SqliteExecutor::Pool(pool) => query.execute(pool).await,
            SqliteExecutor::Transaction(shared_tx) => {
                let mut guard = shared_tx.lock().await;
                if let Some(tx) = guard.as_mut() {
                    query.execute(&mut **tx).await
                } else {
                    Err(sqlx::Error::WorkerCrashed)
                }
            }
        }
    }

    /// Fetches all rows matching the query.
    pub(crate) async fn fetch_all<'q>(&self, query: Query<'q, Sqlite, SqliteArguments<'q>>) -> sqlx::Result<Vec<sqlx::sqlite::SqliteRow>> {
        match self {
            SqliteExecutor::Pool(pool) => query.fetch_all(pool).await,
            SqliteExecutor::Transaction(shared_tx) => {
                let mut guard = shared_tx.lock().await;
                if let Some(tx) = guard.as_mut() {
                    query.fetch_all(&mut **tx).await
                } else {
                    Err(sqlx::Error::WorkerCrashed)
                }
            }
        }
    }
}
