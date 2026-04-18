//! SQLite-specific row and cursor types.
//!
//! - [`SqliteDbCursor`] — async streaming cursor backed by a `BoxStream` of [`sqlx::sqlite::SqliteRow`]
//! - [`SqliteDbRow`] — single row adapter that maps SQLite type affinity to [`DbValue`]

mod row;
mod cursor;

pub use cursor::SqliteDbCursor;