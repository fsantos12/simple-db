use async_trait::async_trait;

use crate::{query::{DeleteQuery, FindQuery, InsertQuery, PreparedDeleteQuery, PreparedFindQuery, PreparedInsertQuery, PreparedUpdateQuery, UpdateQuery}, types::{DbCursor, DbResult}};

/// Executes the four CRUD query types against a database.
///
/// The `prepare_*` methods compile a query into a driver-specific prepared statement.
/// The `find`/`insert`/`update`/`delete` convenience methods prepare and immediately
/// execute in one step.
#[async_trait]
pub trait DbExecutor: Send + Sync {
    /// Compiles a [`FindQuery`] into a prepared statement without executing it.
    fn prepare_find(&self, query: FindQuery) -> DbResult<Box<dyn PreparedFindQuery + '_>>;

    /// Prepares and executes a [`FindQuery`], returning a cursor over the result rows.
    async fn find(&self, query: FindQuery) -> DbResult<Box<dyn DbCursor>>{
        self.prepare_find(query)?.execute().await
    }

    /// Compiles an [`InsertQuery`] into a prepared statement without executing it.
    fn prepare_insert(&self, query: InsertQuery) -> DbResult<Box<dyn PreparedInsertQuery + '_>>;

    /// Prepares and executes an [`InsertQuery`], returning the number of rows inserted.
    async fn insert(&self, query: InsertQuery) -> DbResult<u64> {
        self.prepare_insert(query)?.execute().await
    }

    /// Compiles an [`UpdateQuery`] into a prepared statement without executing it.
    fn prepare_update(&self, query: UpdateQuery) -> DbResult<Box<dyn PreparedUpdateQuery + '_>>;

    /// Prepares and executes an [`UpdateQuery`], returning the number of rows affected.
    async fn update(&self, query: UpdateQuery) -> DbResult<u64> {
        self.prepare_update(query)?.execute().await
    }

    /// Compiles a [`DeleteQuery`] into a prepared statement without executing it.
    fn prepare_delete(&self, query: DeleteQuery) -> DbResult<Box<dyn PreparedDeleteQuery + '_>>;

    /// Prepares and executes a [`DeleteQuery`], returning the number of rows deleted.
    async fn delete(&self, query: DeleteQuery) -> DbResult<u64> {
        self.prepare_delete(query)?.execute().await
    }
}
