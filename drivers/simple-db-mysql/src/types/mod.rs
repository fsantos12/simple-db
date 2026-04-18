//! MySQL-specific row and cursor types.
//!
//! - [`MysqlDbCursor`] — async streaming cursor backed by a `BoxStream` of [`sqlx::mysql::MySqlRow`]
//! - [`MysqlDbRow`] — single row adapter that maps MySQL type names to [`DbValue`]

mod row;
mod cursor;

pub use cursor::MysqlDbCursor;
