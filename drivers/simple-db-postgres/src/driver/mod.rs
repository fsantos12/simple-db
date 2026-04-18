//! PostgreSQL driver and transaction implementations.
//!
//! - [`PostgresDriver`] — pool-backed driver; implements [`DbDriver`](simple_db_core::driver::DbDriver)
//! - [`PostgresTransaction`] — single-connection transaction; implements [`DbTransaction`](simple_db_core::driver::DbTransaction)

mod driver;
mod executor;
mod transaction;

pub use driver::PostgresDriver;
pub use transaction::PostgresTransaction;
