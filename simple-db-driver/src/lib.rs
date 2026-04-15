//! # simple-db-driver
//!
//! Defines the abstract driver interface for the simple-db ecosystem.
//!
//! Consumers depend only on this crate to write backend-agnostic code.
//! Concrete implementations (e.g. SQLite, Postgres) live in separate crates
//! and are injected at runtime via the [`DbDriver`] trait.
//!
//! ## Key exports
//!
//! - [`DbDriver`] — the primary trait every backend must implement
//!
//! ## Re-exports from `simple-db-core`
//!
//! The types below are re-exported here so that driver implementors and callers
//! can import everything they need from a single crate:
//!
//! - [`DbCursor`] — async iterator over query result rows
//! - [`DbRow`] — single result row with typed field access
//! - [`DbRowExt`] — ergonomic extension methods on top of [`DbRow`]
//! - [`DbValue`] — tagged union representing any database value
//! - [`DbError`] — base error trait shared across the ecosystem
//! - [`DbResult`] — `Result<T, Box<dyn DbError>>` alias

use async_trait::async_trait;

use simple_db_query::{DeleteQuery, FindQuery, InsertQuery, UpdateQuery};

// ---------------------------------------------------------------------------
// Public re-exports — stable surface area for driver consumers
// ---------------------------------------------------------------------------

pub use simple_db_core::{DbCursor, DbError, DbResult, DbRow, DbRowExt, DbValue};

// ---------------------------------------------------------------------------
// Trait definition
// ---------------------------------------------------------------------------

/// Abstract async interface for database operations.
///
/// All database interactions flow through this trait, allowing multiple database backends
/// to be swapped without changing application code. Each method is async and can fail,
/// returning `DbError` on problems.
#[async_trait]
pub trait DbDriver: Send + Sync {
    /// Executes a FIND/SELECT query and returns matching rows.
    /// Returns Ok with empty vector if no rows match, not Err(NotFound).
    async fn find(&self, query: FindQuery) -> DbResult<Box<dyn DbCursor>>;

    /// Executes an INSERT query. Returns the number of affected rows.
    async fn insert(&self, query: InsertQuery) -> DbResult<u64>;

    /// Executes an UPDATE query. Returns the number of affected rows.
    async fn update(&self, query: UpdateQuery) -> DbResult<u64>;

    /// Executes a DELETE query. Returns the number of affected rows.
    async fn delete(&self, query: DeleteQuery) -> DbResult<u64>;

    /// Begins a new transaction. Subsequent operations are atomically grouped.
    async fn transaction_begin(&self) -> DbResult<()>;

    /// Commits the current transaction, making all changes permanent.
    async fn transaction_commit(&self) -> DbResult<()>;

    /// Rolls back the current transaction, undoing all changes.
    async fn transaction_rollback(&self) -> DbResult<()>;

    /// Optional: Performs a health check to verify the connection to the backend.
    /// Default implementation is a no-op (always returns Ok).
    async fn ping(&self) ->  DbResult<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_db_core::{DbRow, DbResult};
    use simple_db_query::{DeleteQuery, FindQuery, InsertQuery, UpdateQuery};
    use async_trait::async_trait;

    /// Mock driver implementation for testing
    struct MockDriver;

    #[async_trait]
    impl DbDriver for MockDriver {
        async fn find(&self, _query: FindQuery) -> DbResult<Box<dyn DbCursor>> {
            // Return an empty cursor for testing
            struct EmptyCursor;
            #[async_trait]
            impl DbCursor for EmptyCursor {
                async fn next(&mut self) -> DbResult<Option<Box<dyn DbRow>>> {
                    Ok(None)
                }
            }
            Ok(Box::new(EmptyCursor))
        }

        async fn insert(&self, _query: InsertQuery) -> DbResult<u64> {
            Ok(1)
        }

        async fn update(&self, _query: UpdateQuery) -> DbResult<u64> {
            Ok(1)
        }

        async fn delete(&self, _query: DeleteQuery) -> DbResult<u64> {
            Ok(1)
        }

        async fn transaction_begin(&self) -> DbResult<()> {
            Ok(())
        }

        async fn transaction_commit(&self) -> DbResult<()> {
            Ok(())
        }

        async fn transaction_rollback(&self) -> DbResult<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_driver_find_empty() {
        // Test that find can return empty results
        let driver = MockDriver;
        let query = FindQuery::new("users");
        
        let result = driver.find(query).await;
        assert!(result.is_ok());
        let mut cursor = result.unwrap();
        assert!(cursor.next().await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_driver_insert() {
        // Test insert returns number of affected rows
        let driver = MockDriver;
        let query = InsertQuery::new("users");
        
        let result = driver.insert(query).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }

    #[tokio::test]
    async fn test_driver_update() {
        // Test update returns number of affected rows
        let driver = MockDriver;
        let query = UpdateQuery::new("users");
        
        let result = driver.update(query).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }

    #[tokio::test]
    async fn test_driver_delete() {
        // Test delete returns number of affected rows
        let driver = MockDriver;
        let query = DeleteQuery::new("users");
        
        let result = driver.delete(query).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }

    #[tokio::test]
    async fn test_driver_transaction_begin() {
        // Test transaction begin
        let driver = MockDriver;
        
        let result = driver.transaction_begin().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_driver_transaction_commit() {
        // Test transaction commit
        let driver = MockDriver;
        
        let result = driver.transaction_commit().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_driver_transaction_rollback() {
        // Test transaction rollback
        let driver = MockDriver;
        
        let result = driver.transaction_rollback().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_driver_default_ping() {
        // Test that ping has a default no-op implementation
        let driver = MockDriver;
        
        let result = driver.ping().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_driver_is_send_sync() {
        // Test that Driver is Send + Sync
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<MockDriver>();
    }
}