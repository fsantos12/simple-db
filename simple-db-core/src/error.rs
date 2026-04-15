/// Global error trait for the simple-db ecosystem.
///
/// All crates define their own concrete error types and implement this trait.
/// The universal carrier across APIs is `Box<dyn DbError>`, which is what
/// [`DbResult<T>`](crate::DbResult) uses.
///
/// # Implementing for a custom error type
///
/// ```rust
/// use simple_db_core::DbError;
///
/// #[derive(Debug, thiserror::Error)]
/// pub enum MyDriverError {
///     #[error("connection refused: {0}")]
///     ConnectionRefused(String),
/// }
///
/// impl DbError for MyDriverError {
///     fn message(&self) -> &'static str {
///         match self {
///             Self::ConnectionRefused(_) => "connection refused",
///         }
///     }
/// }
/// ```
pub trait DbError: std::error::Error + Send + Sync {
    /// Returns a static, human-readable category label for this error.
    ///
    /// For full detail (including dynamic fields) use the `Display` impl,
    /// which is available via `.to_string()` or `format!("{}", err)`.
    fn message(&self) -> &'static str;
}

/// Blanket conversion: any concrete `DbError` can be boxed into `Box<dyn DbError>`.
///
/// This makes `?` and `.into()` work transparently when the return type is
/// `Result<_, Box<dyn DbError>>` or [`DbResult<T>`](crate::DbResult).
impl<E: DbError + 'static> From<E> for Box<dyn DbError> {
    fn from(e: E) -> Self {
        Box::new(e)
    }
}

// =============================================================================
// Core error types (value / row access)
// =============================================================================

/// Errors arising from type conversions and row access on [`DbValue`](crate::DbValue)
/// and [`DbRow`](crate::DbRow).
///
/// These are the errors you get when you call `TryFrom<&DbValue>` or the
/// `DbRowExt` helpers and the type or column does not match.
#[derive(Debug, thiserror::Error)]
pub enum TypeError {
    /// The value held a different type than what was requested.
    ///
    /// # Example
    /// Calling `i32::try_from` on a `DbValue` that holds a `String`.
    #[error("type mismatch: expected {expected}, found {found}")]
    Mismatch { expected: String, found: String },

    /// A column index was out of bounds for the row.
    ///
    /// # Example
    /// Calling `row.get_by_index_as::<i32>(99)` on a row with 3 columns.
    #[error("column index out of bounds: {0}")]
    IndexOutOfBounds(usize),

    /// A named column did not exist in the row.
    ///
    /// # Example
    /// Calling `row.get_by_name_as::<String>("email")` when no such column exists.
    #[error("column not found: '{0}'")]
    ColumnMissing(String),
}

impl DbError for TypeError {
    fn message(&self) -> &'static str {
        match self {
            Self::Mismatch { .. } => "type mismatch",
            Self::IndexOutOfBounds(_) => "column index out of bounds",
            Self::ColumnMissing(_) => "column not found",
        }
    }
}
