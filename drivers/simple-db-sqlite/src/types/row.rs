use simple_db_core::types::{DbRow, DbValue};
use sqlx::{Column, Row, TypeInfo, ValueRef, sqlite::SqliteRow};

/// Adapter that wraps a [`SqliteRow`] and exposes it through the [`DbRow`] interface.
///
/// Maps SQLite's four storage classes (INTEGER, REAL, TEXT, BLOB) to the
/// appropriate [`DbValue`] variants. Any unrecognised type is mapped to NULL.
pub struct SqliteDbRow {
    row: SqliteRow,
}

impl SqliteDbRow {
    /// Creates a new adapter wrapping the given raw SQLite row.
    pub fn new(row: SqliteRow) -> Self {
        Self { row }
    }
}

impl DbRow for SqliteDbRow {
    fn get_by_index(&self, index: usize) -> Option<DbValue> {
        let raw_value = self.row.try_get_raw(index).ok()?;
        if raw_value.is_null() { return Some(DbValue::from_null()); }
        match raw_value.type_info().name() {
            "INTEGER" => {
                let val: i64 = self.row.try_get(index).ok()?;
                Some(DbValue::from_i64(val))
            },
            "REAL" => {
                let val: f64 = self.row.try_get(index).ok()?;
                Some(DbValue::from_f64(val))
            },
            "TEXT" => {
                let val: String = self.row.try_get(index).ok()?;
                Some(DbValue::from_string(val))
            },
            "BLOB" => {
                let val: Vec<u8> = self.row.try_get(index).ok()?;
                Some(DbValue::from_bytes(val))
            },
            _ => Some(DbValue::from_null()),
        }
    }

    fn get_by_name(&self, name: &str) -> Option<DbValue> {
        let column = self.row.try_column(name).ok()?;
        self.get_by_index(column.ordinal())
    }

    fn len(&self) -> usize {
        self.row.columns().len()
    }
}