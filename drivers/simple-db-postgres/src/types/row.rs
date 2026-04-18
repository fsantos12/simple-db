use simple_db_core::types::{DbRow, DbValue};
use sqlx::{postgres::PgRow, Column, Row, TypeInfo, ValueRef};

/// Adapter that wraps a [`PgRow`] and exposes it through the [`DbRow`] interface.
///
/// Maps PostgreSQL OID type names to the appropriate [`DbValue`] variants.
/// String-based types (CHAR, TEXT, UUID, JSON, DATE, TIME, TIMESTAMP) are all
/// returned as [`DbValue::from_string`]. Unknown types are mapped to NULL.
pub struct PostgresDbRow {
    row: PgRow,
}

impl PostgresDbRow {
    /// Creates a new adapter wrapping the given raw PostgreSQL row.
    pub fn new(row: PgRow) -> Self {
        Self { row }
    }
}

impl DbRow for PostgresDbRow {
    fn get_by_index(&self, index: usize) -> Option<DbValue> {
        let raw_value = self.row.try_get_raw(index).ok()?;
        if raw_value.is_null() { return Some(DbValue::from_null()); }

        let type_name = raw_value.type_info().name().to_uppercase();
        match type_name.as_str() {
            "INT2" => {
                let val: i16 = self.row.try_get(index).ok()?;
                Some(DbValue::from_i16(val))
            }
            "INT4" => {
                let val: i32 = self.row.try_get(index).ok()?;
                Some(DbValue::from_i32(val))
            }
            "INT8" => {
                let val: i64 = self.row.try_get(index).ok()?;
                Some(DbValue::from_i64(val))
            }
            "FLOAT4" => {
                let val: f32 = self.row.try_get(index).ok()?;
                Some(DbValue::from_f64(val as f64))
            }
            "FLOAT8" | "DOUBLE PRECISION" | "REAL" => {
                let val: f64 = self.row.try_get(index).ok()?;
                Some(DbValue::from_f64(val))
            }
            "BOOL" => {
                let val: bool = self.row.try_get(index).ok()?;
                Some(DbValue::from_bool(val))
            }
            "BYTEA" => {
                let val: Vec<u8> = self.row.try_get(index).ok()?;
                Some(DbValue::from_bytes(val))
            }
            other if other.contains("CHAR")
                || other.contains("TEXT")
                || other.contains("UUID")
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
