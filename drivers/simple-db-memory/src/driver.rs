use std::{collections::HashMap, sync::{Arc, RwLock}};

use async_trait::async_trait;
use simple_db_driver::{types::DbCursor, DbDriver};
use simple_db_query::{queries::{DeleteQuery, FindQuery, InsertQuery, UpdateQuery}, types::{DbError, DbValue}};

#[derive(Default, Clone)]
pub struct MemoryConfig {}

pub struct MemoryDriver {
    config: MemoryConfig,
    storage: Arc<RwLock<HashMap<String, Vec<Vec<(String, DbValue)>>>>>,
}

#[async_trait]
impl DbDriver for MemoryDriver {
    /// Executes a FIND/SELECT query and returns matching rows.
    /// Returns Ok with empty vector if no rows match, not Err(NotFound).
    async fn find(&self, query: FindQuery) -> Result<Box<dyn DbCursor>, DbError> {

    }

    /// Executes an INSERT query. Returns the number of affected rows.
    async fn insert(&self, query: InsertQuery) -> Result<u64, DbError> {
        let mut storage = self.storage.write().unwrap();
        let collection = storage.entry(query.collection).or_default();
        let len = query.values.len() as u64;
        collection.extend(query.values);
        Ok(len)
    }

    /// Executes an UPDATE query. Returns the number of affected rows.
    async fn update(&self, query: UpdateQuery) -> Result<u64, DbError> {
        
    }

    /// Executes a DELETE query. Returns the number of affected rows.
    async fn delete(&self, query: DeleteQuery) -> Result<u64, DbError> {
        
    }

    /// Begins a new transaction. Subsequent operations are atomically grouped.
    async fn transaction_begin(&self) -> Result<(), DbError> {
        Ok(())
    }

    /// Commits the current transaction, making all changes permanent.
    async fn transaction_commit(&self) -> Result<(), DbError> {
        Ok(())
    }

    /// Rolls back the current transaction, undoing all changes.
    async fn transaction_rollback(&self) -> Result<(), DbError> {
        Ok(())
    }
}