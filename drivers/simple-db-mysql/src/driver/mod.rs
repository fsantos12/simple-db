//! MySQL driver and transaction implementations.
//!
//! - [`MysqlDriver`] — pool-backed driver; implements [`DbDriver`](simple_db_core::driver::DbDriver)
//! - [`MysqlTransaction`] — single-connection transaction; implements [`DbTransaction`](simple_db_core::driver::DbTransaction)

mod driver;
mod executor;
mod transaction;

pub use driver::MysqlDriver;
pub use transaction::MysqlTransaction;
