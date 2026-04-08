//! Database error types and context information.
//!
//! Comprehensive error enum covering connection, query, type, concurrency, and
//! mapping failures with detailed context for debugging and recovery.

use std::fmt;
use std::error::Error;

/// Database error types with improved context and chaining support.
#[non_exhaustive] // Prevents breaking changes when adding new variants
#[derive(Debug)]
pub enum DbError {
    /// Database connection failure
    ConnectionError(String),

    /// Syntax or translation error within the Query
    QueryError(String),

    /// Record not found (common in Find operations)
    NotFound,

    /// Type mismatch error with structured expected/found details
    TypeError { expected: String, found: String }, //

    /// Encapsulates native errors from specific drivers (Postgres, Mongo, etc.)
    DriverError(Box<dyn Error + Send + Sync>), //

    /// Concurrency or locking issue (e.g., poisoned lock)
    ConcurrencyError(String),

    /// Mapping error when converting between DbRow and entity models
    MappingError(String),
}

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DbError::ConnectionError(m) => write!(f, "Connection failure: {}", m),
            DbError::QueryError(m) => write!(f, "Query error: {}", m),
            DbError::NotFound => write!(f, "Record not found"),
            DbError::TypeError { expected, found } => {
                write!(f, "Type mismatch: expected {}, found {}", expected, found)
            }
            DbError::DriverError(e) => write!(f, "Driver-specific error: {}", e),
            DbError::ConcurrencyError(m) => write!(f, "Concurrency/Locking issue: {}", m),
            DbError::MappingError(m) => write!(f, "Mapping failure: {}", m),
        }
    }
}

impl Error for DbError {
    /// Allows walking the error chain to find the underlying cause.
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            DbError::DriverError(e) => Some(e.as_ref()),
            _ => None,
        }
    }
}

/// Helper implementation to allow using '?' with standard IO errors.
impl From<std::io::Error> for DbError {
    fn from(err: std::io::Error) -> Self {
        DbError::DriverError(Box::new(err))
    }
}