//! Core driver interface and transaction handling.
//!
//! The `Driver` trait defines the abstraction layer for database backends.
//! Implementations (Memory, PostgreSQL, MongoDB, etc.) provide concrete query
//! execution and transaction management logic.

use std::sync::Arc;
use std::future::Future;

use async_trait::async_trait;

use crate::{types::{DbError, DbRow}, query::{DeleteQuery, FindQuery, InsertQuery, UpdateQuery}};

/// The core interface for database backends.
/// Keep this trait "Object Safe" by avoiding generic methods here.
#[async_trait]
pub trait Driver: Send + Sync {
    // --- Queries ---
    async fn find(&self, query: FindQuery) -> Result<Vec<DbRow>, DbError>;
    async fn insert(&self, query: InsertQuery) -> Result<u64, DbError>;
    async fn update(&self, query: UpdateQuery) -> Result<u64, DbError>;
    async fn delete(&self, query: DeleteQuery) -> Result<u64, DbError>;

    // --- Transactional Hooks ---
    async fn transaction_begin(&self) -> Result<(), DbError>;
    async fn transaction_commit(&self) -> Result<(), DbError>;
    async fn transaction_rollback(&self) -> Result<(), DbError>;

    /// Optional: Performs a health check to verify the connection to the backend.
    async fn ping(&self) -> Result<(), DbError> {
        Ok(())
    }
}

/// Extension for Arc<dyn Driver> to provide the high-level Transaction API.
/// This allows the transactional logic to be reused across all drivers.
#[async_trait]
pub trait Transactional {
    async fn transaction<F, Fut, T>(&self, f: F) -> Result<T, DbError>
    where
        F: FnOnce(Arc<dyn Driver>) -> Fut + Send,
        Fut: Future<Output = Result<T, DbError>> + Send,
        T: Send;
}

#[async_trait]
impl Transactional for Arc<dyn Driver> {
    async fn transaction<F, Fut, T>(&self, f: F) -> Result<T, DbError>
    where F: FnOnce(Arc<dyn Driver>) -> Fut + Send, Fut: Future<Output = Result<T, DbError>> + Send, T: Send {
        // 1. Start the transaction context in the driver
        self.transaction_begin().await?;

        // 2. Execute the business logic passing a clone of the Arc handle
        // We clone 'self' (the Arc) so the closure can use the same driver instance.
        let result = f(self.clone()).await;

        match result {
            Ok(value) => {
                // 3. Success: Commit changes permanently
                self.transaction_commit().await?;
                Ok(value)
            }
            Err(err) => {
                // 4. Failure: Rollback to ensure Atomicity (ACID)
                // We ignore the rollback error to return the original logic error
                let _ = self.transaction_rollback().await;
                Err(err)
            }
        }
    }
}
