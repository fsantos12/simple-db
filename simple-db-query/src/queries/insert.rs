use crate::types::DbValue;

/// INSERT query builder for adding data to a collection.
///
/// Supports:
/// - Single row insertion via `.insert()`
/// - Bulk insertion via `.bulk_insert()` for multiple rows
/// - Automatic type conversion via `Into<DbValue>`
///
/// # Example
///
/// ```rust,ignore
/// // Insert a single row
/// let query = Query::insert("users")
///     .insert(vec![
///         ("name", "Alice"),
///         ("email", "alice@example.com"),
///         ("age", 30i32),
///     ]);
///
/// // Bulk insert multiple rows
/// let query = Query::insert("users")
///     .insert(vec![("name", "Alice"), ("email", "alice@example.com")])
///     .insert(vec![("name", "Bob"), ("email", "bob@example.com")]);
/// ```
/// 
/// # Column/Value Format
///
/// Rows are specified as iterables of `(column_name, value)` pairs:
/// - Column names: `&str` or `String`
/// - Values: implement `Into<DbValue>` (i32, f64, String, etc.)
#[derive(Debug, Clone)]
pub struct InsertQuery {
    pub collection: String,
    pub values: Vec<Vec<(String, DbValue)>>,
}

impl InsertQuery {
    /// Creates a new insert query for the given collection.
    pub fn new<S: Into<String>>(collection: S) -> Self {
        Self {
            collection: collection.into(),
            values: Vec::new(),
        }
    }

    /// Inserts a single row. Values are automatically converted to DbValue.
    /// Multiple calls append additional rows.
    pub fn insert<I, K, V>(mut self, row: I) -> Self
    where I: IntoIterator<Item = (K, V)>, K: Into<String>, V: Into<DbValue> {
        let db_row: Vec<(String, DbValue)> = row.into_iter().map(|(k, v)| (k.into(), v.into())).collect();
        self.values.push(db_row);
        self
    }

    /// Batch inserts multiple rows at once. Each row is an iterable of (column, value) pairs.
    pub fn bulk_insert<I, R, K, V>(mut self, rows: I) -> Self
    where I: IntoIterator<Item = R>, R: IntoIterator<Item = (K, V)>, K: Into<String>, V: Into<DbValue> {
        for row in rows {
            let db_row: Vec<(String, DbValue)> = row.into_iter().map(|(k, v)| (k.into(), v.into())).collect();
            self.values.push(db_row);
        }
        self
    }
}