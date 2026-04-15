//! # Simple-DB-Query: Type-Safe Database Query Engine
//!
//! A comprehensive, high-performance database query builder for Rust with:
//! - **Type-safe query construction** via fluent builder pattern
//! - **Efficient value representation** using bit-packed 64-bit `DbValue`
//! - **Async/await support** for non-blocking database operations
//! - **Zero-copy where possible** with SmallVec stack allocation
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! // Find users older than 18
//! let query = Query::find("users")
//!     .project(|b| b.field("name").field("email"))
//!     .filter(|b| b.gt("age", 18))
//!     .order_by(|b| b.asc("name"))
//!     .limit(10);
//! ```
//!
//! ## Core Modules
//!
//! - **[types]**: Value encoding, errors
//! - **[queries]**: FindQuery, InsertQuery, UpdateQuery, DeleteQuery
//! - **[builders]**: Fluent query components (filters, projections, sorts, groups)

pub mod types;
pub mod builders;
pub mod queries;