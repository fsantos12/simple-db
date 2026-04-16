//! # simple-db-core
//!
//! Foundation layer for the simple-db ecosystem.
//!
//! Provides the shared primitives used by all sibling crates:
//!
//! - [`DbValue`] — compact 64-bit tagged union covering all database value types
//! - [`DbRow`] / [`DbRowExt`] — traits for row-level value access and type conversion
//! - [`DbCursor`] — async streaming cursor over query results
//! - [`DbError`] — unified error enum for all simple-db operations
//! - [`DbResult<T>`](DbResult) — `Result<T, DbError>` alias used throughout

mod error;
mod row;
mod cursor;
mod value;

pub use error::DbError;
pub use row::{DbRow, DbRowExt};
pub use cursor::DbCursor;
pub use value::DbValue;

// =============================================================================
// Result alias
// =============================================================================

/// Shorthand for `Result<T, DbError>`.
///
/// Used as the return type for every fallible operation in the simple-db
/// ecosystem.
pub type DbResult<T> = Result<T, DbError>;

