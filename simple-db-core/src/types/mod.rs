//! Shared types for value storage, row access, cursor streaming, and error handling.
//!
//! All public types from this module are re-exported at the crate root for convenience:
//! - [`DbValue`] — compact 64-bit tagged union for any SQL value
//! - [`DbRow`] / [`DbRowExt`] — traits for accessing columns by index or name
//! - [`DbCursor`] — async iterator over query result rows
//! - [`DbError`] — unified error type
//! - [`DbResult`] — `Result<T, DbError>` type alias

mod error;
mod value;
mod row;
mod cursor;

pub use error::DbError;
pub use value::DbValue;
pub use row::{DbRow, DbRowExt};
pub use cursor::DbCursor;

// =============================================================================
// Result alias
// =============================================================================
/// Shorthand for `Result<T, DbError>`.
///
/// Used as the return type for every fallible operation in the simple-db
/// ecosystem.
pub type DbResult<T> = Result<T, DbError>;