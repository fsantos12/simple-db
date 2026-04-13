//! Insert query builder for adding records to a collection.

use std::collections::HashMap;

use crate::types::DbValue;

/// A row of key-value pairs to be inserted or updated.
pub type DataRow = HashMap<String, DbValue>;

/// Add one or more new records to a collection.
#[derive(Debug, Clone)]
pub struct InsertQuery {
    pub collection: String,
    pub values: Vec<DataRow>,
}

impl InsertQuery {
    pub fn new<S: Into<String>>(collection: S) -> Self {
        Self {
            collection: collection.into(),
            values: Vec::new(),
        }
    }

    /// Inserts a single row from a collection of key-value pairs.
    pub fn insert<I, K, V>(mut self, row: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<DbValue>,
    {
        let data_row: DataRow = row.into_iter().map(|(k, v)| (k.into(), v.into())).collect();
        self.values.push(data_row);
        self
    }

    /// Batch inserts multiple rows efficiently.
    pub fn bulk_insert<I, R, K, V>(mut self, rows: I) -> Self
    where
        I: IntoIterator<Item = R>,
        R: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<DbValue>,
    {
        for row in rows {
            let data_row: DataRow = row.into_iter().map(|(k, v)| (k.into(), v.into())).collect();
            self.values.push(data_row);
        }
        self
    }

    /// Directly inserts pre-built `DataRow` objects.
    pub fn values(mut self, rows: Vec<DataRow>) -> Self {
        self.values.extend(rows);
        self
    }
}
