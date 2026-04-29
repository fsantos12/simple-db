use std::sync::Arc;
use tokio::sync::Mutex;
use sqlx::{MySql, MySqlPool, Transaction, mysql::{MySqlArguments, MySqlQueryResult}, query::Query};

/// Executes sqlx queries against either a connection pool or an active transaction.
///
/// Internally wraps either a `MySqlPool` for general query execution or an
/// `Arc<Mutex<Option<Transaction<MySql>>>>` for executing within a transaction.
/// This enum allows query builders to remain agnostic about the execution context.
pub(crate) enum MySqlExecutor {
    /// Executes queries against the connection pool.
    Pool(MySqlPool),
    /// Executes queries against an active transaction.
    Transaction(Arc<Mutex<Option<Transaction<'static, MySql>>>>),
}

impl MySqlExecutor {
    /// Executes a query and returns the result (affected row count, last insert id, etc.).
    pub(crate) async fn execute(&self, query: Query<'_, MySql, MySqlArguments>) -> sqlx::Result<MySqlQueryResult> {
        match self {
            MySqlExecutor::Pool(pool) => query.execute(pool).await,
            MySqlExecutor::Transaction(shared_tx) => {
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
    pub(crate) async fn fetch_all(&self, query: Query<'_, MySql, MySqlArguments>) -> sqlx::Result<Vec<sqlx::mysql::MySqlRow>> {
        match self {
            MySqlExecutor::Pool(pool) => query.fetch_all(pool).await,
            MySqlExecutor::Transaction(shared_tx) => {
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
