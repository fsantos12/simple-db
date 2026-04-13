use async_trait::async_trait;

use crate::{queries::{DeleteQuery, FindQuery, InsertQuery, UpdateQuery}, types::{DbError, DbRow}};

/// Abstract async interface for database operations.
///
/// All database interactions flow through this trait, allowing multiple database backends
/// to be swapped without changing application code. Each method is async and can fail,
/// returning `DbError` on problems.
#[async_trait]
pub trait Driver: Send + Sync {
    /// Executes a SELECT query and returns matching rows.
    /// Returns Ok with empty vector if no rows match, not Err(NotFound).
    async fn find(&self, query: FindQuery) -> Result<Vec<Box<dyn DbRow>>, DbError>;

    /// Executes an INSERT query. Returns the number of affected rows.
    async fn insert(&self, query: InsertQuery) -> Result<u64, DbError>;

    /// Executes an UPDATE query. Returns the number of affected rows.
    async fn update(&self, query: UpdateQuery) -> Result<u64, DbError>;

    /// Executes a DELETE query. Returns the number of affected rows.
    async fn delete(&self, query: DeleteQuery) -> Result<u64, DbError>;

    /// Begins a new transaction. Subsequent operations are atomically grouped.
    async fn transaction_begin(&self) -> Result<(), DbError>;

    /// Commits the current transaction, making all changes permanent.
    async fn transaction_commit(&self) -> Result<(), DbError>;

    /// Rolls back the current transaction, undoing all changes.
    async fn transaction_rollback(&self) -> Result<(), DbError>;

    /// Optional: Performs a health check to verify the connection to the backend.
    /// Default implementation is a no-op (always returns Ok).
    async fn ping(&self) -> Result<(), DbError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::queries::{FindQuery, InsertQuery, UpdateQuery, DeleteQuery};
    use async_trait::async_trait;

    /// Mock driver implementation for testing
    struct MockDriver;

    #[async_trait]
    impl Driver for MockDriver {
        async fn find(&self, _query: FindQuery) -> Result<Vec<Box<dyn DbRow>>, DbError> {
            Ok(Vec::new())
        }

        async fn insert(&self, _query: InsertQuery) -> Result<u64, DbError> {
            Ok(1)
        }

        async fn update(&self, _query: UpdateQuery) -> Result<u64, DbError> {
            Ok(1)
        }

        async fn delete(&self, _query: DeleteQuery) -> Result<u64, DbError> {
            Ok(1)
        }

        async fn transaction_begin(&self) -> Result<(), DbError> {
            Ok(())
        }

        async fn transaction_commit(&self) -> Result<(), DbError> {
            Ok(())
        }

        async fn transaction_rollback(&self) -> Result<(), DbError> {
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
        assert_eq!(result.unwrap().len(), 0);
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