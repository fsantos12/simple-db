//! # simple-db-postgres
//!
//! PostgreSQL backend driver for the simple-db ecosystem.
//!
//! Implements [`DbDriver`](simple_db_core::driver::DbDriver) and
//! [`DbTransaction`](simple_db_core::driver::DbTransaction) on top of
//! [`sqlx::PgPool`](sqlx::PgPool).
//!
//! ## Usage
//!
//! ```rust,ignore
//! use std::sync::Arc;
//! use simple_db_core::DbContext;
//! use simple_db_postgres::PostgresDriver;
//! use sqlx::postgres::PgPoolOptions;
//!
//! let pool = PgPoolOptions::new()
//!     .connect("postgres://user:pass@localhost:5432/mydb")
//!     .await?;
//! let ctx = DbContext::new(Arc::new(PostgresDriver::new(pool)));
//! ```

mod types;
mod builders;
mod queries;
mod driver;

pub use driver::{PostgresDriver, PostgresTransaction};
