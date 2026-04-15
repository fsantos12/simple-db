//! # simple-db-query
//!
//! Type-safe, driver-agnostic query builder for the simple-db ecosystem.
//!
//! ## Quick Start
//!
//! ```rust
//! use simple_db_query::Query;
//!
//! // SELECT with filters, sorts, and pagination
//! let q = Query::find("users")
//!     .project(|b| b.field("name").field("email"))
//!     .filter(|b| b.gt("age", 18i32))
//!     .order_by(|b| b.asc("name"))
//!     .limit(10);
//!
//! // INSERT
//! let q = Query::insert("users")
//!     .insert(vec![("name", "Alice"), ("email", "alice@example.com")]);
//!
//! // UPDATE
//! let q = Query::update("users")
//!     .set("active", false)
//!     .filter(|b| b.lt("last_login_days", 90i32));
//!
//! // DELETE
//! let q = Query::delete("users")
//!     .filter(|b| b.eq("archived", true));
//! ```
//!
//! ## Modules
//!
//! - [`queries`] — [`Query`] entry point and CRUD query types
//! - [`builders`] — filter, projection, sort, and group-by builders

mod builders;
mod queries;

// Re-export the most commonly used types at the crate root for convenience.
pub use queries::{Query, FindQuery, InsertQuery, UpdateQuery, DeleteQuery};
pub use builders::{
    Filter, FilterBuilder, FilterDefinition,
    Projection, ProjectionBuilder, ProjectionDefinition,
    Sort, SortBuilder, SortDefinition,
    GroupBuilder, GroupDefinition,
};
