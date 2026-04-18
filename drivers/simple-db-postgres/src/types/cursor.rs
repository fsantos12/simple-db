use futures::{stream::BoxStream, StreamExt};
use simple_db_core::types::{DbCursor, DbError, DbResult};
use sqlx::postgres::PgRow;

use crate::types::row::PostgresDbRow;

/// Async cursor that streams [`PgRow`] values from a query result set.
///
/// Implements [`DbCursor`] so that consumers can iterate rows in a database-agnostic way.
pub struct PostgresDbCursor {
    stream: BoxStream<'static, Result<PgRow, sqlx::Error>>,
}

impl PostgresDbCursor {
    /// Creates a new cursor from a pinned [`BoxStream`] of PostgreSQL rows.
    pub fn new(stream: BoxStream<'static, Result<PgRow, sqlx::Error>>) -> Self {
        Self { stream }
    }
}

#[async_trait::async_trait]
impl DbCursor for PostgresDbCursor {
    async fn next(&mut self) -> DbResult<Option<Box<dyn simple_db_core::types::DbRow>>> {
        match self.stream.next().await {
            Some(Ok(pg_row)) => Ok(Some(Box::new(PostgresDbRow::new(pg_row)))),
            Some(Err(err)) => Err(DbError::driver(err)),
            None => Ok(None),
        }
    }
}
