//! **Error types for database operations**
//!
//! This module defines the error hierarchy used throughout the database query engine.
//! Errors are organized into three main categories:
//!
//! - **`DbError`**: Public API error, wraps other error types via `#[from]`
//! - **`TypeError`**: Type conversion and value access errors
//! - **`QueryError`**: Query construction and syntax errors
//! - **`DriverError`**: Database driver/connection errors
//!
//! All errors implement `std::error::Error` and can be used with `?` operator.

use std::error::Error;
use thiserror::Error;

// ============================================================================
// PRIMARY ERROR TYPE (Public API Interface)
// ============================================================================

/// The primary error type for database operations.
///
/// `DbError` serves as the public error type that can wrap any of the specialized
/// error categories via the `#[from]` attribute. This allows seamless error
/// conversion and a unified error handling interface.
///
/// # Variants
///
/// - `Type`: Type conversion or value access failed
/// - `Query`: Query construction or syntax error
/// - `Driver`: Database driver or connection error
/// - `NotFound`: Record not found (common ORM use case)
///
/// # Example
///
/// ```rust
/// use simple_db_query::types::{DbError, TypeError};
///
/// // Errors automatically convert
/// let type_err = TypeError::IndexOutOfBounds(5);
/// let db_err: DbError = type_err.into();
///
/// // Pattern matching
/// match db_err {
///     DbError::Type(e) => println!("Type error: {}", e),
///     DbError::NotFound => println!("Record not found"),
///     _ => println!("Other error"),
/// }
/// ```
#[derive(Debug, Error)]
pub enum DbError {
    /// Type conversion, value access, or type checking failed.
    /// Transparently delegates to `TypeError` with automatic conversion.
    #[error(transparent)]
    Type(#[from] TypeError),

    /// Query construction, validation, or syntax error.
    /// Transparently delegates to `QueryError` with automatic conversion.
    #[error(transparent)]
    Query(#[from] QueryError),

    /// Database driver error: connection, execution, or transaction failure.
    /// Transparently delegates to `DriverError` with automatic conversion.
    #[error(transparent)]
    Driver(#[from] DriverError),

    /// Record not found.
    /// This is placed here because it's the most common logical error in ORM operations.
    #[error("record not found")]
    NotFound,
}

// ============================================================================
// TYPE ERROR (Value conversion and type checking)
// ============================================================================

/// Errors related to type conversion and value access.
///
/// Occurs when:
/// - A value is accessed with the wrong type
/// - A column index is out of bounds
/// - A column name doesn't exist
#[derive(Debug, Error)]
pub enum TypeError {
    /// Type mismatch when converting a value.
    ///
    /// Happens when trying to extract a value as a different type than it contains.
    /// For example: trying to get an i32 from a String value.
    ///
    /// # Example
    /// ```text
    /// type mismatch: expected i32, found String
    /// ```
    #[error("type mismatch: expected {expected}, found {found}")]
    Mismatch { expected: String, found: String },

    /// Row index is out of bounds.
    ///
    /// Occurs when accessing a column by index that doesn't exist.
    ///
    /// # Example
    /// ```text
    /// index out of bounds: 5
    /// ```
    #[error("index out of bounds: {0}")]
    IndexOutOfBounds(usize),

    /// Column name not found in row.
    ///
    /// Occurs when accessing a column by name that doesn't exist.
    ///
    /// # Example
    /// ```text
    /// missing column: 'user_email'
    /// ```
    #[error("missing column: '{0}'")]
    ColumnMissing(String),
}

// ============================================================================
// QUERY ERROR (Query construction and validation)
// ============================================================================

/// Errors related to query construction, validation, and syntax.
///
/// Occurs during:
/// - Query builder operations
/// - Query syntax validation
/// - Invalid argument values
#[derive(Debug, Error)]
pub enum QueryError {
    /// Query construction failed.
    ///
    /// Occurs when building a query fails due to invalid state,
    /// missing required parameters, or logical errors.
    ///
    /// # Example
    /// ```text
    /// failed to build query: missing WHERE clause
    /// ```
    #[error("failed to build query: {0}")]
    Build(String),

    /// SQL syntax error.
    ///
    /// Occurs when the query has invalid SQL syntax or grammar.
    ///
    /// # Example
    /// ```text
    /// syntax error: unexpected token 'FROM' at position 15
    /// ```
    #[error("syntax error: {0}")]
    Syntax(String),

    /// Invalid argument value or type.
    ///
    /// Occurs when a query argument has an invalid value,
    /// type, or format.
    ///
    /// # Example
    /// ```text
    /// invalid argument: limit must be positive
    /// ```
    #[error("invalid argument: {0}")]
    InvalidArgument(String),
}

// ============================================================================
// DRIVER ERROR (Database connection and execution)
// ============================================================================

/// Errors from the underlying database driver.
///
/// These are typically IO/network errors or errors from the database itself.
/// The source error provides additional context from the driver.
#[derive(Debug, Error)]
pub enum DriverError {
    /// Database connection failed.
    ///
    /// Occurs during connection establishment or when the connection is lost.
    /// The boxed source error contains driver-specific details.
    ///
    /// # Example
    /// ```text
    /// database connection failed: (PostgreSQL error details)
    /// ```
    #[error("database connection failed")]
    Connection(#[source] Box<dyn Error + Send + Sync + 'static>),

    /// Query execution failed.
    ///
    /// Occurs when the database rejects or fails to execute a query.
    /// The boxed source error contains driver-specific details.
    ///
    /// # Example
    /// ```text
    /// query execution failed: (PostgreSQL error details)
    /// ```
    #[error("query execution failed")]
    Execution(#[source] Box<dyn Error + Send + Sync + 'static>),

    /// Transaction error.
    ///
    /// Occurs during transaction commit, rollback, or other transaction operations.
    /// The boxed source error contains driver-specific details.
    ///
    /// # Example
    /// ```text
    /// transaction error: (PostgreSQL error details)
    /// ```
    #[error("transaction error")]
    Transaction(#[source] Box<dyn Error + Send + Sync + 'static>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_error_conversion() {
        // Test that TypeError automatically converts to DbError
        let type_err = TypeError::IndexOutOfBounds(5);
        let _db_err: DbError = type_err.into();
        // Conversion should work without error
    }

    #[test]
    fn test_query_error_conversion() {
        // Test that QueryError automatically converts to DbError
        let query_err = QueryError::InvalidArgument("bad value".to_string());
        let _db_err: DbError = query_err.into();
        // Conversion should work without error
    }

    #[test]
    fn test_driver_error_conversion() {
        // Test that DriverError automatically converts to DbError
        use std::error::Error;
        let driver_err = DriverError::Connection(
            Box::<dyn Error + Send + Sync>::from("timeout")
        );
        let _db_err: DbError = driver_err.into();
        // Conversion should work without error
    }

    #[test]
    fn test_not_found_error() {
        // Test NotFound variant
        let not_found = DbError::NotFound;
        assert!(matches!(not_found, DbError::NotFound));
    }

    #[test]
    fn test_error_display() {
        // Test that errors have helpful Display messages
        let err = DbError::NotFound;
        let msg = format!("{}", err);
        assert!(!msg.is_empty());
    }

    #[test]
    fn test_type_mismatch_error() {
        let err = TypeError::Mismatch {
            expected: "i32".to_string(),
            found: "String".to_string(),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("i32"));
        assert!(msg.contains("String"));
    }

    #[test]
    fn test_column_missing_error() {
        let err = TypeError::ColumnMissing("user_email".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("user_email"));
    }
}