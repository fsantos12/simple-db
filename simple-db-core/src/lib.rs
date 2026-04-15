//! # simple-db-core
//!
//! Foundation layer for the simple-db ecosystem.
//!
//! Provides the shared primitives used by all sibling crates:
//!
//! - [`DbValue`] — compact 64-bit tagged union covering all database value types
//! - [`DbRow`] / [`DbRowExt`] — traits for row-level value access and type conversion
//! - [`DbCursor`] — async streaming cursor over query results
//! - [`DbError`] — global error trait implemented by every crate's concrete error types
//! - [`TypeError`] — concrete error for value/row type mismatches
//! - [`DbResult<T>`](DbResult) — `Result<T, Box<dyn DbError>>` alias used throughout

// ---------------------------------------------------------------------------
// Private modules — implementation details, not part of the public API
// ---------------------------------------------------------------------------

mod error;
mod value;
mod row;
mod cursor;
mod result;

// ---------------------------------------------------------------------------
// Public re-exports — the stable surface area of this crate
// ---------------------------------------------------------------------------

// Error types
pub use error::{DbError, TypeError};

// Value type
pub use value::DbValue;

// Row traits
pub use row::{DbRow, DbRowExt};

// Cursor trait
pub use cursor::DbCursor;

// Result alias
pub use result::DbResult;
