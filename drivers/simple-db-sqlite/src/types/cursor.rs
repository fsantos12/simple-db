use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use simple_db_core::types::{DbCursor, DbError, DbResult};
use sqlx::sqlite::SqliteRow;

use crate::types::row::SqliteDbRow;

/// Async cursor that streams [`SqliteRow`] values from a query result set.
///
/// Implements [`DbCursor`] so that consumers can iterate rows in a database-agnostic way.
pub struct SqliteDbCursor {
    stream: BoxStream<'static, Result<SqliteRow, sqlx::Error>>,
}

impl SqliteDbCursor {
    /// Creates a new cursor from a pinned [`BoxStream`] of SQLite rows.
    pub fn new(stream: BoxStream<'static, Result<SqliteRow, sqlx::Error>>) -> Self {
        Self { stream }
    }
}

#[async_trait]
impl DbCursor for SqliteDbCursor {
    async fn next(&mut self) -> DbResult<Option<Box<dyn simple_db_core::types::DbRow>>> {
        match self.stream.next().await {
            Some(Ok(sqlite_row)) => Ok(Some(Box::new(SqliteDbRow::new(sqlite_row)))),
            Some(Err(e))        => Err(DbError::driver(e)),
            None                => Ok(None),
        }
    }
}
