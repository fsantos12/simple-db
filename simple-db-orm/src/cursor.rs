use async_trait::async_trait;
use simple_db_core::types::{DbCursor, DbResult};

use crate::entity::DbEntityTrait;

/// Extension trait for converting cursor rows into typed entities.
///
/// This trait adds convenience methods to the core `DbCursor` type for hydrating
/// entities directly from query results without manual `DbRow` to entity conversion.
#[async_trait]
pub trait DbCursorEntityExt {
    /// Fetches and converts the next row into an entity of type `T`.
    ///
    /// Returns `Ok(Some(entity))` if a row exists, `Ok(None)` if the cursor is exhausted,
    /// or `Err` if a database error occurs.
    async fn next_entity<T: DbEntityTrait>(&mut self) -> DbResult<Option<T>>;
}

#[async_trait]
impl<C: DbCursor + ?Sized> DbCursorEntityExt for C {
    async fn next_entity<T: DbEntityTrait>(&mut self) -> DbResult<Option<T>> {
        if let Some(row) = self.next().await? {
            Ok(Some(T::from_db(row.as_ref())))
        } else {
            Ok(None)
        }
    }
}
