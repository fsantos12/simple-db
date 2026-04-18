//! # simple-db-core
//!
//! Core abstractions for the simple-db ecosystem.
//!
//! This crate defines the database driver traits, the query builder types,
//! and all shared types (values, rows, cursors, errors) that every backend
//! and application layer depends on.
//!
//! ## Crate structure
//!
//! - [`context`] — [`DbContext`]: high-level entry point that wraps a [`driver::DbDriver`]
//! - [`driver`] — [`driver::DbExecutor`], [`driver::DbDriver`], [`driver::DbTransaction`] trait hierarchy
//! - [`query`] — type-safe query builders (find / insert / update / delete)
//! - [`types`] — shared value, row, cursor, and error types
//!
//! ## Quick example
//!
//! ```rust,ignore
//! use simple_db_core::{DbContext, query::Query};
//! use std::sync::Arc;
//!
//! let ctx = DbContext::new(Arc::new(my_driver));
//! let mut cursor = ctx.find(Query::find("users").filter(|b| b.eq("active", true))).await?;
//! while let Some(row) = cursor.next().await? {
//!     // process row...
//! }
//! ```

pub mod context;
pub mod driver;
pub mod query;
pub mod types;

pub use context::DbContext;