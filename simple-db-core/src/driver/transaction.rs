use std::sync::Arc;
use core::future::Future;

use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::{driver::{driver::DbDriver, executor::DbExecutor}, types::{DbError, DbResult}};

/// A database transaction supporting ACID semantics.
///
/// A transaction is an executor that can be committed or rolled back atomically.
/// Most applications use the [`DbTransactionExt::transaction`] helper method rather than
/// directly managing this trait.
#[async_trait]
pub trait DbTransaction: DbExecutor {
    /// Commits the transaction, applying all changes to the database.
    async fn commit(&self) -> DbResult<()>;
    
    /// Rolls back the transaction, discarding all changes.
    async fn rollback(&self) -> DbResult<()>;
}

/// Extension trait providing convenience methods on drivers to run transactional blocks.
#[async_trait]
pub trait DbTransactionExt {
    /// Runs the closure `f` in a new transaction, committing on success and rolling back on error.
    ///
    /// # Example
    /// ```ignore
    /// let driver: Arc<dyn DbDriver> = todo!();
    /// let result = driver.transaction(|tx| async {
    ///     // Perform queries using tx...
    ///     Ok(value)
    /// }).await?;
    /// ```
    async fn transaction<F, Fut, T>(&self, f: F) -> DbResult<T>
    where F: FnOnce(Arc<dyn DbTransaction>) -> Fut + Send, Fut: Future<Output = DbResult<T>> + Send, T: Send;
}

/// Backend-neutral helper for safely extracting and executing an action on a shared transaction.
///
/// This generic function handles the Mutex acquire, Option extraction, and error wrapping
/// needed across all driver implementations.
pub async fn close_transaction<T, F, Fut, E>(
    shared_tx: &Arc<Mutex<Option<T>>>,
    closed_error: &'static str,
    action: F,
) -> DbResult<()>
where
    T: Send,
    F: FnOnce(T) -> Fut + Send,
    Fut: Future<Output = Result<(), E>> + Send,
    E: std::error::Error + Send + Sync + 'static,
{
    let mut guard = shared_tx.lock().await;
    let tx = match guard.take() {
        Some(tx) => tx,
        None => return Err(DbError::Internal(closed_error.into())),
    };

    drop(guard);
    action(tx).await.map_err(DbError::driver)
}

#[async_trait]
impl DbTransactionExt for Arc<dyn DbDriver> {
    async fn transaction<F, Fut, T>(&self, f: F) -> DbResult<T>
    where F: FnOnce(Arc<dyn DbTransaction>) -> Fut + Send, Fut: Future<Output = DbResult<T>> + Send, T: Send, {
        let tx = self.begin().await?;
        let result = f(tx.clone()).await;
        match result {
            Ok(value) => {
                tx.commit().await?;
                Ok(value)
            }
            Err(e) => {
                let _ = tx.rollback().await;
                Err(e)
            }
        }
    }
}
