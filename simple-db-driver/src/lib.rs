//! # simple-db-driver
//!
//! Defines the abstract driver interface for the simple-db ecosystem.
//!
//! Consumers depend only on this crate to write backend-agnostic code.
//! Concrete implementations (e.g. SQLite, Postgres) live in separate crates
//! and are injected at runtime via [`DbDriver`].
//!
//! ## Trait hierarchy
//!
//! ```text
//! DbExecutor          (find / insert / update / delete)
//! ├── DbDriver        : DbExecutor  (+begin, +ping)
//! └── DbTransaction   : DbExecutor  (+commit, +rollback)
//! ```
//!
//! ## Key exports
//!
//! - [`DbExecutor`] — shared CRUD interface; accept this in functions that do not need transactions
//! - [`DbDriver`] — pool-level driver; starts transactions via [`DbDriver::begin`]
//! - [`DbTransaction`] — pinned-connection transaction handle
//! - [`DbTransactionExt`] — extension on `Arc<dyn DbDriver>` for managed transactions (auto commit/rollback)
//!
//! ## Re-exports from `simple-db-core`
//!
//! - [`DbCursor`], [`DbRow`], [`DbRowExt`], [`DbValue`], [`DbError`], [`DbResult`]

pub mod executor;
pub mod driver;
pub mod transaction;

pub use executor::DbExecutor;
pub use driver::DbDriver;
pub use transaction::{DbTransaction, DbTransactionExt};

// Convenience re-exports so driver implementors only need this one crate.
pub use simple_db_core::{DbCursor, DbRow, DbRowExt, DbValue, DbError, DbResult};

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;

    use simple_db_core::{DbCursor, DbResult, DbRow};
    use simple_db_query::{DeleteQuery, FindQuery, InsertQuery, UpdateQuery};

    use super::*;

    // ---- Mock transaction --------------------------------------------------

    struct MockTransaction;

    #[async_trait]
    impl DbExecutor for MockTransaction {
        async fn find(&self, _query: FindQuery) -> DbResult<Box<dyn DbCursor>> {
            Ok(Box::new(EmptyCursor))
        }
        async fn insert(&self, _query: InsertQuery) -> DbResult<u64> { Ok(1) }
        async fn update(&self, _query: UpdateQuery) -> DbResult<u64> { Ok(1) }
        async fn delete(&self, _query: DeleteQuery) -> DbResult<u64> { Ok(1) }
    }

    #[async_trait]
    impl DbTransaction for MockTransaction {
        async fn commit(&self) -> DbResult<()> { Ok(()) }
        async fn rollback(&self) -> DbResult<()> { Ok(()) }
    }

    // ---- Mock driver -------------------------------------------------------

    struct MockDriver;

    #[async_trait]
    impl DbExecutor for MockDriver {
        async fn find(&self, _query: FindQuery) -> DbResult<Box<dyn DbCursor>> {
            Ok(Box::new(EmptyCursor))
        }
        async fn insert(&self, _query: InsertQuery) -> DbResult<u64> { Ok(1) }
        async fn update(&self, _query: UpdateQuery) -> DbResult<u64> { Ok(1) }
        async fn delete(&self, _query: DeleteQuery) -> DbResult<u64> { Ok(1) }
    }

    #[async_trait]
    impl DbDriver for MockDriver {
        async fn begin(&self) -> DbResult<Arc<dyn DbTransaction>> {
            Ok(Arc::new(MockTransaction))
        }
    }

    // ---- Helpers -----------------------------------------------------------

    struct EmptyCursor;

    #[async_trait]
    impl DbCursor for EmptyCursor {
        async fn next(&mut self) -> DbResult<Option<Box<dyn DbRow>>> {
            Ok(None)
        }
    }

    // ---- Tests -------------------------------------------------------------

    #[tokio::test]
    async fn test_executor_find_returns_empty_cursor() {
        let driver = MockDriver;
        let mut cursor = driver.find(FindQuery::new("users")).await.unwrap();
        assert!(cursor.next().await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_executor_insert_returns_affected_rows() {
        assert_eq!(MockDriver.insert(InsertQuery::new("users")).await.unwrap(), 1);
    }

    #[tokio::test]
    async fn test_executor_update_returns_affected_rows() {
        assert_eq!(MockDriver.update(UpdateQuery::new("users")).await.unwrap(), 1);
    }

    #[tokio::test]
    async fn test_executor_delete_returns_affected_rows() {
        assert_eq!(MockDriver.delete(DeleteQuery::new("users")).await.unwrap(), 1);
    }

    #[tokio::test]
    async fn test_driver_ping_default_is_ok() {
        assert!(MockDriver.ping().await.is_ok());
    }

    #[tokio::test]
    async fn test_driver_begin_returns_transaction() {
        let driver = MockDriver;
        let tx = driver.begin().await;
        assert!(tx.is_ok());
    }

    #[tokio::test]
    async fn test_transaction_commit() {
        assert!(MockTransaction.commit().await.is_ok());
    }

    #[tokio::test]
    async fn test_transaction_rollback() {
        assert!(MockTransaction.rollback().await.is_ok());
    }

    #[tokio::test]
    async fn test_transaction_executes_queries() {
        let tx = MockTransaction;
        assert_eq!(tx.insert(InsertQuery::new("users")).await.unwrap(), 1);
        assert_eq!(tx.update(UpdateQuery::new("users")).await.unwrap(), 1);
        assert_eq!(tx.delete(DeleteQuery::new("users")).await.unwrap(), 1);
    }

    #[tokio::test]
    async fn test_managed_transaction_commits_on_success() {
        let driver: Arc<dyn DbDriver> = Arc::new(MockDriver);
        let result = driver.transaction(|tx| async move {
            tx.insert(InsertQuery::new("users")).await?;
            Ok(42u32)
        }).await;
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_managed_transaction_rolls_back_on_error() {
        let driver: Arc<dyn DbDriver> = Arc::new(MockDriver);
        let result: DbResult<()> = driver.transaction(|_tx| async move {
            Err(DbError::ColumnNotFound("id".into()))
        }).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_driver_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<MockDriver>();
        assert_send_sync::<MockTransaction>();
    }
}
