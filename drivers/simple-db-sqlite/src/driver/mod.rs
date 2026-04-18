//! SQLite driver and transaction implementations.
//!
//! - [`SqliteDriver`] — pool-backed driver; implements [`DbDriver`](simple_db_core::driver::DbDriver)
//! - [`SqliteTransaction`] — single-connection transaction; implements [`DbTransaction`](simple_db_core::driver::DbTransaction)

mod driver;
mod executor;
mod transaction;

pub use driver::SqliteDriver;
pub use transaction::SqliteTransaction;
