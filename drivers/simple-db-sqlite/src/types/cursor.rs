use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use simple_db_core::{DbCursor, DbError, DbRow, DbResult};
use sqlx::sqlite::SqliteRow;

use crate::types::row::SqliteDbRow;

pub struct SqliteDbCursor {
    stream: BoxStream<'static, Result<SqliteRow, sqlx::Error>>,
}

impl SqliteDbCursor {
    pub fn new(stream: BoxStream<'static, Result<SqliteRow, sqlx::Error>>) -> Self {
        Self { stream }
    }
}

#[async_trait]
impl DbCursor for SqliteDbCursor {
    async fn next(&mut self) -> DbResult<Option<Box<dyn DbRow>>> {
        match self.stream.next().await {
            Some(Ok(sqlite_row)) => Ok(Some(Box::new(SqliteDbRow::new(sqlite_row)))),
            Some(Err(e))        => Err(DbError::driver(e)),
            None                => Ok(None),
        }
    }
}
