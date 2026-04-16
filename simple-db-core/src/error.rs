/// Unified error type for the simple-db ecosystem.
///
/// All fallible operations return `DbResult<T>` which is `Result<T, DbError>`.
/// Driver implementors wrap their native errors in the [`DbError::Driver`] variant.
///
/// # Example
///
/// ```rust
/// use simple_db_core::{DbError, DbResult};
///
/// fn fail() -> DbResult<i32> {
///     Err(DbError::TypeMismatch { expected: "i32".into(), found: "String".into() })
/// }
///
/// assert!(fail().is_err());
/// ```
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    // --- Type / row access errors ---

    /// The value held a different type than what was requested.
    ///
    /// # Example
    /// Calling `i32::try_from` on a `DbValue` that holds a `String`.
    #[error("type mismatch: expected {expected}, found {found}")]
    TypeMismatch { expected: String, found: String },

    /// A column index was out of bounds for the row.
    ///
    /// # Example
    /// Calling `row.get_by_index_as::<i32>(99)` on a row with 3 columns.
    #[error("column index out of bounds: {0}")]
    ColumnIndexOutOfBounds(usize),

    /// A named column did not exist in the row.
    ///
    /// # Example
    /// Calling `row.get_by_name_as::<String>("email")` when no such column exists.
    #[error("column not found: '{0}'")]
    ColumnNotFound(String),

    // --- Driver / IO errors ---

    /// An error originating from an underlying database driver or IO layer.
    ///
    /// Use [`DbError::driver`] to construct this variant from any error type.
    #[error("driver error: {0}")]
    Driver(#[source] Box<dyn std::error::Error + Send + Sync>),
}

impl DbError {
    /// Wraps any error into the [`DbError::Driver`] variant.
    ///
    /// # Example
    /// ```rust,ignore
    /// .map_err(DbError::driver)?;
    /// ```
    pub fn driver(e: impl std::error::Error + Send + Sync + 'static) -> Self {
        DbError::Driver(Box::new(e))
    }
}