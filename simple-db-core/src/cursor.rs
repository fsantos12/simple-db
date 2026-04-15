//! Async streaming cursor over query results.
//!
//! A [`DbCursor`] represents an open query result set that can be iterated
//! row by row. Rows are streamed lazily, so large result sets do not need to
//! be fully loaded into memory.

use async_trait::async_trait;

use crate::{DbResult, DbRow};

/// Async iterator over the rows returned by a query.
///
/// Drivers implement this trait to expose their native result sets through a
/// common interface. Consumers call [`next`](DbCursor::next) in a loop until
/// it returns `Ok(None)`, signalling that all rows have been consumed.
///
/// # Example
///
/// ```rust
/// # use simple_db_core::{DbCursor, DbResult};
/// # async fn consume(mut cursor: Box<dyn DbCursor>) -> DbResult<()> {
/// while let Some(row) = cursor.next().await? {
///     // process row…
/// }
/// # Ok(())
/// # }
/// ```
#[async_trait]
pub trait DbCursor: Send {
    /// Advances the cursor and returns the next row, or `Ok(None)` when
    /// the result set is exhausted.
    async fn next(&mut self) -> DbResult<Option<Box<dyn DbRow>>>;
}
