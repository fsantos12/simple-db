use futures::{stream::BoxStream, StreamExt};
use simple_db_core::types::{DbCursor, DbError, DbResult};
use sqlx::mysql::MySqlRow;

use crate::types::row::MysqlDbRow;

/// Async cursor that streams [`MySqlRow`] values from a query result set.
///
/// Implements [`DbCursor`] so that consumers can iterate rows in a database-agnostic way.
pub struct MysqlDbCursor {
    stream: BoxStream<'static, Result<MySqlRow, sqlx::Error>>,
}

impl MysqlDbCursor {
    /// Creates a new cursor from a pinned [`BoxStream`] of MySQL rows.
    pub fn new(stream: BoxStream<'static, Result<MySqlRow, sqlx::Error>>) -> Self {
        Self { stream }
    }
}

#[async_trait::async_trait]
impl DbCursor for MysqlDbCursor {
    async fn next(&mut self) -> DbResult<Option<Box<dyn simple_db_core::types::DbRow>>> {
        match self.stream.next().await {
            Some(Ok(row)) => Ok(Some(Box::new(MysqlDbRow::new(row)))),
            Some(Err(err)) => Err(DbError::driver(err)),
            None => Ok(None),
        }
    }
}
