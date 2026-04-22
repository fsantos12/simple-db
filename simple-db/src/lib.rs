//! # simple-db
//!
//! Umbrella crate that re-exports the simple-db ecosystem in one place.
//!
//! Enable the driver(s) you need via feature flags:
//! - `sqlite` — SQLite backend via [`SqliteDriver`] / [`SqliteTransaction`]
//! - `postgres` — PostgreSQL backend via [`PostgresDriver`] / [`PostgresTransaction`]
//! - `mysql` — MySQL backend via [`MysqlDriver`] / [`MysqlTransaction`]
//!
//! ## Example
//!
//! ```toml
//! [dependencies]
//! simple-db = { version = "*", features = ["sqlite"] }
//! ```
//!
//! ```rust,ignore
//! use std::sync::Arc;
//! use simple_db::{DbContext, SqliteDriver};
//!
//! let ctx = DbContext::new(Arc::new(SqliteDriver::new(pool)));
//! ```

pub use simple_db_core::DbContext;
pub use simple_db_core::driver;
pub use simple_db_core::query;
pub use simple_db_core::types;
pub use simple_db_core::{filter, project, sort, group};

#[cfg(feature = "orm")]
pub use simple_db_orm::{DbEntity, DbEntityTrait, TrackingState};

#[cfg(feature = "orm")]
pub use simple_db_macros::DbEntity as DeriveDbEntity;

#[cfg(feature = "sqlite")]
pub use simple_db_sqlite::{SqliteDriver, SqlitePool, SqlitePoolOptions, SqliteTransaction};

#[cfg(feature = "postgres")]
pub use simple_db_postgres::{PgPool, PgPoolOptions, PostgresDriver, PostgresTransaction};

#[cfg(feature = "mysql")]
pub use simple_db_mysql::{MySqlPool, MySqlPoolOptions, MysqlDriver, MysqlTransaction};
