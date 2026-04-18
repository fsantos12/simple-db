use simple_db_core::types::{DbRow, DbValue};
use sqlx::{mysql::MySqlRow, Column, Row, TypeInfo, ValueRef};

/// Adapter that wraps a [`MySqlRow`] and exposes it through the [`DbRow`] interface.
///
/// Maps MySQL type names to the appropriate [`DbValue`] variants.
/// String-based types (CHAR, TEXT, JSON, DATE, TIME, TIMESTAMP) are all
/// returned as [`DbValue::from_string`]. Unknown types are mapped to NULL.
pub struct MysqlDbRow {
    row: MySqlRow,
}

impl MysqlDbRow {
    /// Creates a new adapter wrapping the given raw MySQL row.
    pub fn new(row: MySqlRow) -> Self {
        Self { row }
    }
}

impl DbRow for MysqlDbRow {
    fn get_by_index(&self, index: usize) -> Option<DbValue> {
        let raw_value = self.row.try_get_raw(index).ok()?;
        if raw_value.is_null() { return Some(DbValue::from_null()); }

        let type_name = raw_value.type_info().name().to_uppercase();
        match type_name.as_str() {
            "TINYINT" | "SMALLINT" | "MEDIUMINT" | "INT" | "BIGINT" => {
                let val: i64 = self.row.try_get(index).ok()?;
                Some(DbValue::from_i64(val))
            }
            "FLOAT" | "DOUBLE" | "DECIMAL" => {
                let val: f64 = self.row.try_get(index).ok()?;
                Some(DbValue::from_f64(val))
            }
            "BOOL" | "BOOLEAN" => {
                let val: bool = self.row.try_get(index).ok()?;
                Some(DbValue::from_bool(val))
            }
            "BLOB" | "TINYBLOB" | "MEDIUMBLOB" | "LONGBLOB" | "VARBINARY" | "BINARY" => {
                let val: Vec<u8> = self.row.try_get(index).ok()?;
                Some(DbValue::from_bytes(val))
            }
            other if other.contains("CHAR")
                || other.contains("TEXT")
                || other.contains("JSON")
                || other.contains("DATE")
                || other.contains("TIME")
                || other.contains("TIMESTAMP") => {
                let val: String = self.row.try_get(index).ok()?;
                Some(DbValue::from_string(val))
            }
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
