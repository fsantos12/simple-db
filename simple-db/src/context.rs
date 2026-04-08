//! Database context serving as the central entry point for all database operations.
//!
//! `DbContext` abstracts the underlying driver implementation and provides a clean
//! API for queries (Find, Insert, Update, Delete), transactions, and entity hydration.
//! It manages the unit of work pattern and change tracking for ORM operations.

use std::sync::Arc;

use crate::{driver::driver::{Driver, Transactional}, entity::{DbEntity, DbEntityModel}, query::{DeleteQuery, FindQuery, InsertQuery, UpdateQuery}, types::{DbError, DbRow}};

pub struct DbContext {
    driver: Arc<dyn Driver>
}

impl DbContext {
    pub fn new(driver: Arc<dyn Driver>) -> Self {
        Self { driver }
    }

    // --- Transaction Management ---
    pub async fn transaction<F, Fut, T>(&self, f: F) -> Result<T, DbError>
    where F: FnOnce(Arc<dyn Driver>) -> Fut + Send, Fut: std::future::Future<Output = Result<T, DbError>> + Send, T: Send {
        self.driver.transaction(f).await
    }

    // --- Queries ---
    pub async fn find(&self, query: FindQuery) -> Result<Vec<DbRow>, DbError> {
        self.driver.find(query).await
    }

    pub async fn insert(&self, query: InsertQuery) -> Result<u64, DbError>{
        self.driver.insert(query).await
    }

    pub async fn update(&self, query: UpdateQuery) -> Result<u64, DbError>{
        self.driver.update(query).await
    }

    pub async fn delete(&self, query: DeleteQuery) -> Result<u64, DbError>{
        self.driver.delete(query).await
    }

    // --- Entity Hydration (ORM Layer) ---
    /// Executes a find query and automatically wraps each row into a tracked DbEntity.
    /// This initializes the entity in the 'Tracked' state with a snapshot of its current data.
    pub async fn find_entities<T: DbEntityModel>(&self, query: FindQuery) -> Result<Vec<DbEntity<T>>, DbError> {
        // 1. Fetch raw rows from the driver
        let rows = self.find(query).await?;
        let mut entities = Vec::with_capacity(rows.len());

        // 2. Map each row to a tracked entity
        for row in rows {
            // We use the raw row both to build the model and as the initial snapshot
            let model = T::from_db_row(row.clone())?;
            entities.push(DbEntity::from_db(model, row));
        }

        Ok(entities)
    }

    /// Executes a find query and deserializes rows directly to entities (read-only).
    /// Returns entities without ORM tracking, suitable for read-only queries.
    /// This is more memory-efficient than find_entities when you don't need change tracking.
    pub async fn find_entities_readonly<T: DbEntityModel>(&self, query: FindQuery) -> Result<Vec<T>, DbError> {
        let rows = self.find(query).await?;
        let mut entities = Vec::with_capacity(rows.len());

        for row in rows {
            entities.push(T::from_db_row(row)?);
        }

        Ok(entities)
    }
}