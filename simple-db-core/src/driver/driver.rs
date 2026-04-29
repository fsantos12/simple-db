use std::sync::Arc;

use async_trait::async_trait;

use crate::{driver::{executor::DbExecutor, transaction::DbTransaction}, types::DbResult};

/// The top-level database driver.
///
/// A [`DbDriver`] extends [`DbExecutor`] with connection-level operations: beginning a
/// transaction and pinging the server. Wrap it in an [`Arc`] and pass it to
/// [`DbContext`](crate::DbContext) to make it available to application code.
#[async_trait]
pub trait DbDriver: DbExecutor {
    /// Begins a new database transaction and returns a handle to it.
    async fn begin(&self) -> DbResult<Arc<dyn DbTransaction>>;

    /// Checks the connection to the database server. Default implementation always succeeds.
    async fn ping(&self) -> DbResult<()> { Ok(()) }
}
