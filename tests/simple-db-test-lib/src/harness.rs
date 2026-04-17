use async_trait::async_trait;
use simple_db::DbContext;

/// Implemented by each driver test app to provide a fresh, schema-ready DbContext.
///
/// `create_context` must drop and recreate all tables so that every test and
/// benchmark starts with a completely empty database.
#[async_trait]
pub trait TestHarness: Send + Sync {
    fn driver_name(&self) -> &str;
    async fn create_context(&self) -> DbContext;
}
