//! **Type System Module**
//!
//! Core type definitions for the database query engine:
//!
//! - **`DbValue`**: Efficient tagged union for all SQL value types
//! - **`DbRow`**: Trait for generic row access by index or column name
//! - **Error Types**: Comprehensive error hierarchy for type, query, and driver errors
//!
//! # Example
//!
//! ```rust
//! use simple_db_query::types::{DbValue, DbError};
//!
//! // Create a string value
//! let value = DbValue::from_string("hello");
//! assert_eq!(value.as_string(), Some("hello"));
//!
//! // Type conversion with error handling
//! let num: Result<i32, DbError> = value.try_into();
//! assert!(num.is_err()); // Can't convert string to i32
//! ```

mod error;
mod value;
mod row;

pub use error::{TypeError, QueryError, DriverError, DbError};
pub use value::DbValue;
pub use row::{DbRow, DbRowExt};