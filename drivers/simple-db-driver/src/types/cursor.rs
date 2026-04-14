use async_trait::async_trait;
use simple_db_query::types::{DbError, DbRow};

#[async_trait]
pub trait DbCursor: Send {
    async fn next(&mut self) -> Result<Option<Box<dyn DbRow>>, DbError>;
}