//! # simple-db
//!
//! A type-safe, async-first query builder and ORM for Rust with in-memory and pluggable database drivers.
//!
//! ## Features
//!
//! - **Type-Safe Queries**: Fluent API with builder pattern for constructing queries
//! - **Async/Await**: Native Tokio integration for async database operations
//! - **Change Tracking**: Automatic entity state management (Added, Tracked, Deleted, Detached)
//! - **Flexible Filtering**: Rich filter API supporting comparisons, ranges, patterns, and logical operators
//! - **Memory Driver**: In-memory database implementation for testing and prototyping
//! - **Driver Abstraction**: Pluggable architecture for adding new database backends
//!
//! ## Quick Start
//!
//! ```ignore
//! use simple_db::{DbContext, driver::memory::MemoryDriver, query::Query};
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() {
//!     let driver = Arc::new(MemoryDriver::new());
//!     let ctx = DbContext::new(driver);
//!
//!     // Find users older than 18
//!     let query = Query::find("users")
//!         .filter(|fb| fb.gte("age", 18));
//!
//!     let results = ctx.find(query).await.unwrap();
//! }
//! ```

pub mod types;
pub mod query;
pub mod driver;
pub mod entity;

mod context;

pub use context::DbContext;
pub use entity::{DbEntity, DbEntityModel, DbEntityState};