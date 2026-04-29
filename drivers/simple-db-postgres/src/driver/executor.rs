use std::sync::Arc;
use tokio::sync::Mutex;
use sqlx::{Postgres, PgPool, Transaction, postgres::{PgArguments, PgQueryResult}, query::Query};

/// Executes sqlx queries against either a connection pool or an active transaction.
///
/// Internally wraps either a `PgPool` for general query execution or an
/// `Arc<Mutex<Option<Transaction<Postgres>>>>` for executing within a transaction.
/// This enum allows query builders to remain agnostic about the execution context.
pub(crate) enum PostgresExecutor {
    /// Executes queries against the connection pool.
    Pool(PgPool),
    /// Executes queries against an active transaction.
    Transaction(Arc<Mutex<Option<Transaction<'static, Postgres>>>>),
}

impl PostgresExecutor {
    /// Executes a query and returns the result (affected row count, etc.).
    pub(crate) async fn execute(&self, query: Query<'_, Postgres, PgArguments>) -> sqlx::Result<PgQueryResult> {
        match self {
            PostgresExecutor::Pool(pool) => query.execute(pool).await,
            PostgresExecutor::Transaction(shared_tx) => {
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
    pub(crate) async fn fetch_all(&self, query: Query<'_, Postgres, PgArguments>) -> sqlx::Result<Vec<sqlx::postgres::PgRow>> {
        match self {
            PostgresExecutor::Pool(pool) => query.fetch_all(pool).await,
            PostgresExecutor::Transaction(shared_tx) => {
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
