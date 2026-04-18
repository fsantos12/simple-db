//! PostgreSQL-specific row and cursor types.
//!
//! - [`PostgresDbCursor`] — async streaming cursor backed by a `BoxStream` of [`sqlx::postgres::PgRow`]
//! - [`PostgresDbRow`] — single row adapter that maps PostgreSQL OID types to [`DbValue`]

mod row;
mod cursor;

pub use cursor::PostgresDbCursor;
