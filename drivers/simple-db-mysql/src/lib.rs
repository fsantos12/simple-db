//! # simple-db-mysql
//!
//! MySQL backend driver for the simple-db ecosystem.
//!
//! Implements [`DbDriver`](simple_db_core::driver::DbDriver) and
//! [`DbTransaction`](simple_db_core::driver::DbTransaction) on top of
//! [`sqlx::MySqlPool`](sqlx::MySqlPool).
//!
//! ## Usage
//!
//! ```rust,ignore
//! use std::sync::Arc;
//! use simple_db_core::DbContext;
//! use simple_db_mysql::MysqlDriver;
//! use sqlx::mysql::MySqlPoolOptions;
//!
//! let pool = MySqlPoolOptions::new()
//!     .connect("mysql://root:root@localhost:3306/mydb")
//!     .await?;
//! let ctx = DbContext::new(Arc::new(MysqlDriver::new(pool)));
//! ```

mod types;
mod builders;
mod queries;
mod driver;

pub use driver::{MysqlDriver, MysqlTransaction};
