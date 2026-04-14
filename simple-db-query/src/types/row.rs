//! **Row trait for generic database row access**
//!
//! This module defines `DbRow`, a trait that abstracts over different row implementations
//! while providing a unified interface for accessing column values.
//!
//! Implementers can provide rows from different sources:
//! - In-memory result sets
//! - Streaming query results
//! - Test fixtures

use crate::types::{DbValue, TypeError, DbError};

/// Generic trait for accessing database row values (object-safe).
///
/// `DbRow` abstracts over different row implementations, allowing code to work
/// with rows from any source (in-memory, streaming, etc.) via a common interface.
/// This trait is object-safe and can be used as `dyn DbRow`.
///
/// For type-safe conversions, use the [`DbRowExt`] extension trait.
///
/// # Access Methods
///
/// Values can be accessed:
/// - **By index**: `get_by_index(0)` for the first column
/// - **By name**: `get_by_name("email")` for a named column
///
/// # Example
///
/// ```rust
/// # use simple_db_query::types::{DbValue, DbRow};
/// #
/// // Implementer provides a row
/// // let row: &dyn DbRow = ...;
/// // let value = row.get_by_index(0);
/// // let named = row.get_by_name("email");
/// ```
pub trait DbRow: Send + Sync {
    /// Returns a reference to the value at the given column index.
    ///
    /// Returns `None` if the index is out of bounds.
    ///
    /// # Arguments
    /// * `index` - 0-based column index
    ///
    /// # Example
    /// ```text
    /// let value = row.get_by_index(0)?; // First column
    /// ```
    fn get_by_index(&self, index: usize) -> Option<&DbValue>;

    /// Returns a reference to the value in the column with the given name.
    ///
    /// Returns `None` if no column with that name exists.
    ///
    /// # Arguments
    /// * `name` - Column name (case-sensitive)
    ///
    /// # Example
    /// ```text
    /// let value = row.get_by_name("user_id")?;
    /// ```
    fn get_by_name(&self, name: &str) -> Option<&DbValue>;

    /// Returns the number of columns in this row.
    ///
    /// # Example
    /// ```text
    /// assert!(row.len() > 0);
    /// ```
    fn len(&self) -> usize;

    /// Returns true if this row has no columns.
    ///
    /// # Example
    /// ```text
    /// assert!(!row.is_empty());
    /// ```
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Extension trait for type-safe value conversions on `DbRow`.
///
/// This trait provides generic methods for converting row values to specific types.
/// It's automatically available for all types implementing `DbRow`.
///
/// # Example
///
/// ```rust
/// # use simple_db_query::types::{DbRow, DbRowExt};
/// # fn example(row: &dyn DbRow) -> Result<(), Box<dyn std::error::Error>> {
/// let user_id: i32 = row.get_by_index_as(0)?;
/// let email: String = row.get_by_name_as("email")?;
/// # Ok(())
/// # }
/// ```
pub trait DbRowExt: DbRow {
    /// Returns the value at the given column index, converted to type `T`.
    ///
    /// Provides type-safe access with automatic error conversion.
    ///
    /// # Type Parameters
    /// * `T` - Target type that implements `TryFrom<&DbValue>`
    ///
    /// # Errors
    /// * `TypeError::IndexOutOfBounds` - Index doesn't exist
    /// * Any error from `T::try_from` - Type conversion failed
    ///
    /// # Example
    /// ```rust
    /// # use simple_db_query::types::{DbRow, DbRowExt};
    /// # fn example(row: &dyn DbRow) -> Result<(), Box<dyn std::error::Error>> {
    /// let user_id: i32 = row.get_by_index_as(0)?;
    /// let email: String = row.get_by_index_as(3)?;
    /// # Ok(())
    /// # }
    /// ```
    fn get_by_index_as<'a, T>(&'a self, index: usize) -> Result<T, DbError>
    where T: TryFrom<&'a DbValue, Error = DbError> {
        let value = self.get_by_index(index).ok_or_else(|| TypeError::IndexOutOfBounds(index))?;
        T::try_from(value)
    }

    /// Returns the value in the named column, converted to type `T`.
    ///
    /// Provides type-safe access by column name with automatic error conversion.
    ///
    /// # Type Parameters
    /// * `T` - Target type that implements `TryFrom<&DbValue>`
    ///
    /// # Errors
    /// * `TypeError::ColumnMissing` - Column name doesn't exist
    /// * Any error from `T::try_from` - Type conversion failed
    ///
    /// # Example
    /// ```rust
    /// # use simple_db_query::types::{DbRow, DbRowExt};
    /// # fn example(row: &dyn DbRow) -> Result<(), Box<dyn std::error::Error>> {
    /// let name: String = row.get_by_name_as("user_name")?;
    /// let age: i32 = row.get_by_name_as("age")?;
    /// # Ok(())
    /// # }
    /// ```
    fn get_by_name_as<'a, T>(&'a self, name: &str) -> Result<T, DbError>
    where T: TryFrom<&'a DbValue, Error = DbError> {
        let value = self.get_by_name(name).ok_or_else(|| TypeError::ColumnMissing(name.to_string()))?;
        T::try_from(value)
    }
}

/// Blanket implementation: all `DbRow` types automatically implement `DbRowExt`.
impl<T: DbRow + ?Sized> DbRowExt for T {}