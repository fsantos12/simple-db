//! Core type definitions for database values, rows, and error handling.
//!
//! This module provides foundational types: `DbValue` for typed database values,
//! `DbRow` for representing table rows, `DbError` for error cases, and traits
//! for converting between entity models and database rows.

mod error;
mod value;
mod row;

pub use error::DbError;
pub use value::DbValue;
pub use row::DbRow;
pub use row::FromDbRow;