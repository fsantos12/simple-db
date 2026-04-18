//! # simple-db-sqlite
//!
//! SQLite backend driver for the simple-db ecosystem.
//!
//! Implements [`DbDriver`](simple_db_core::driver::DbDriver) and
//! [`DbTransaction`](simple_db_core::driver::DbTransaction) on top of
//! [`sqlx::SqlitePool`](sqlx::SqlitePool).
//!
//! ## Usage
//!
//! ```rust,ignore
//! use std::sync::Arc;
//! use simple_db_core::DbContext;
//! use simple_db_sqlite::SqliteDriver;
//! use sqlx::sqlite::SqlitePoolOptions;
//!
//! let pool = SqlitePoolOptions::new()
//!     .connect("sqlite://:memory:")
//!     .await?;
//! let ctx = DbContext::new(Arc::new(SqliteDriver::new(pool)));
//! ```

mod types;
mod builders;
mod queries;
mod driver;

pub use driver::{SqliteDriver, SqliteTransaction};