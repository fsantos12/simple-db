use std::{collections::HashMap, sync::{Arc, RwLock}};

use async_trait::async_trait;
use simple_db_driver::Driver;
use simple_db_query::{queries::{DeleteQuery, FindQuery, InsertQuery, UpdateQuery}, types::{DbError, DbRow, DriverError}};

use crate::types::MemoryRow;

#[derive(Default, Clone)]
pub struct MemoryConfig {}

pub struct MemoryDriver {
    config: MemoryConfig,
    storage: Arc<RwLock<HashMap<String, Vec<MemoryRow>>>>
}

#[async_trait]
impl Driver for MemoryDriver {
    /// Executes a FIND/SELECT query and returns matching rows.
    /// Returns Ok with empty vector if no rows match, not Err(NotFound).
    async fn find(&self, query: FindQuery) -> Result<Vec<Box<dyn DbRow>>, DbError> {

    }

    /// Executes an INSERT query. Returns the number of affected rows.
    async fn insert(&self, query: InsertQuery) -> Result<u64, DbError> {
        let mut storage = self.storage.write().unwrap();
        let collection = storage.entry(query.collection).or_default();
        let len = query.values.len() as u64;

        query.values.into_iter().for_each(|row| {
            
        });

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
        
    }

    /// Commits the current transaction, making all changes permanent.
    async fn transaction_commit(&self) -> Result<(), DbError> {
        
    }

    /// Rolls back the current transaction, undoing all changes.
    async fn transaction_rollback(&self) -> Result<(), DbError> {
        
    }
}