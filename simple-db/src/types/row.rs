//! Database row representation and type-safe accessors.
//!
//! `DbRow` provides a HashMap-backed representation of database rows with
//! helper methods for type-safe field access and conversion to entity models.

use std::collections::HashMap;

use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use serde_json::Value as JsonValue;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::types::{DbError, value::DbValue};

#[derive(Debug, Clone, Default)]
pub struct DbRow(pub HashMap<String, DbValue>);

macro_rules! impl_type_helpers {
    // Branch para tipos Boxed (ex: String, Json)
    ($suffix:ident, $variant:ident, $type:ty, boxed) => {
        paste::paste! {
            /// Get a reference to a boxed field. Rust's deref coercion handles &Box<T> -> &T.
            pub fn [<get_ $suffix>](&self, key: &str) -> Result<&$type, DbError> {
                match self.get(key) {
                    Some(DbValue::$variant(Some(v))) => Ok(v), // Deref coercion automatically works here
                    Some(DbValue::$variant(None)) => Err(DbError::MappingError(format!("Field '{}' is NULL", key))),
                    Some(other) => Err(DbError::TypeError { 
                        expected: stringify!($variant).to_string(), 
                        found: format!("{:?}", other) 
                    }),
                    None => Err(DbError::NotFound),
                }
            }

            /// Takes ownership and automatically unboxes the value.
            pub fn [<take_ $suffix>](&mut self, key: &str) -> Result<$type, DbError> {
                match self.take(key) {
                    Some(DbValue::$variant(Some(v))) => Ok(*v), // The '*' unboxes the value
                    Some(DbValue::$variant(None)) => Err(DbError::MappingError(format!("Field '{}' is NULL", key))),
                    Some(other) => Err(DbError::TypeError { 
                        expected: stringify!($variant).to_string(), 
                        found: format!("{:?}", other) 
                    }),
                    None => Err(DbError::NotFound),
                }
            }
        }
    };

    // Branch para tipos simples na Stack (ex: i32, bool)
    ($suffix:ident, $variant:ident, $type:ty) => {
        paste::paste! {
            pub fn [<get_ $suffix>](&self, key: &str) -> Result<&$type, DbError> {
                match self.get(key) {
                    Some(DbValue::$variant(Some(v))) => Ok(v),
                    Some(DbValue::$variant(None)) => Err(DbError::MappingError(format!("Field '{}' is NULL", key))),
                    Some(other) => Err(DbError::TypeError { 
                        expected: stringify!($variant).to_string(), 
                        found: format!("{:?}", other) 
                    }),
                    None => Err(DbError::NotFound),
                }
            }

            pub fn [<take_ $suffix>](&mut self, key: &str) -> Result<$type, DbError> {
                match self.take(key) {
                    Some(DbValue::$variant(Some(v))) => Ok(v),
                    _ => Err(DbError::MappingError(format!("Invalid or missing field '{}'", key))),
                }
            }
        }
    };
}


impl DbRow {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Insert a value into the row
    pub fn insert<K: Into<String>, V: Into<DbValue>>(&mut self, key: K, value: V) {
        self.0.insert(key.into(), value.into());
    }

    /// Get a reference to a value
    pub fn get(&self, key: &str) -> Option<&DbValue> {
        self.0.get(key)
    }

    /// Take ownership of a value (removes it from the row)
    /// This is very efficient for mapping to models
    pub fn take(&mut self, key: &str) -> Option<DbValue> {
        self.0.remove(key)
    }

    // Implement type-specific helper methods for common types
    // Primitive types
    impl_type_helpers!(i8, I8, i8);
    impl_type_helpers!(i16, I16, i16);
    impl_type_helpers!(i32, I32, i32);
    impl_type_helpers!(i64, I64, i64);
    impl_type_helpers!(i128, I128, i128);
    impl_type_helpers!(u8, U8, u8);
    impl_type_helpers!(u16, U16, u16);
    impl_type_helpers!(u32, U32, u32);
    impl_type_helpers!(u64, U64, u64);
    impl_type_helpers!(u128, U128, u128);
    impl_type_helpers!(f32, F32, f32);
    impl_type_helpers!(f64, F64, f64);
    impl_type_helpers!(bool, Bool, bool);
    impl_type_helpers!(char, Char, char);

    // Temporal types
    impl_type_helpers!(date, Date, NaiveDate);
    impl_type_helpers!(time, Time, NaiveTime);
    impl_type_helpers!(timestamp, Timestamp, NaiveDateTime);
    impl_type_helpers!(timestamptz, Timestamptz, DateTime<Utc>);

    // Large types (boxed for efficiency)
    impl_type_helpers!(decimal, Decimal, Decimal, boxed);
    impl_type_helpers!(string, String, String, boxed);
    impl_type_helpers!(bytes, Bytes, Vec<u8>, boxed);
    impl_type_helpers!(uuid, Uuid, Uuid, boxed);
    impl_type_helpers!(json, Json, JsonValue, boxed);
}

// This allows: .collect::<DbRow>()
impl FromIterator<(String, DbValue)> for DbRow {
    fn from_iter<I: IntoIterator<Item = (String, DbValue)>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl IntoIterator for DbRow {
    type Item = (String, DbValue);
    type IntoIter = std::collections::hash_map::IntoIter<String, DbValue>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

pub trait FromDbRow: Sized {
    fn from_db_row(row: DbRow) -> Result<Self, DbError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_row_new() {
        let row = DbRow::new();
        assert_eq!(row.0.len(), 0);
    }

    #[test]
    fn test_row_insert_and_get() {
        let mut row = DbRow::new();
        row.insert("age", 42i32);
        row.insert("name", "Alice");
        row.insert("active", true);

        assert_eq!(row.get("age"), Some(&DbValue::I32(Some(42))));
        assert_eq!(
            row.get("name"),
            Some(&DbValue::String(Some(Box::new("Alice".to_string()))))
        );
        assert_eq!(row.get("active"), Some(&DbValue::Bool(Some(true))));
    }

    #[test]
    fn test_row_get_returns_none_for_missing_keys() {
        let row = DbRow::new();
        assert_eq!(row.get("nonexistent"), None);
    }

    #[test]
    fn test_row_take() {
        let mut row = DbRow::new();
        row.insert("name", "Bob");
        row.insert("age", 30i32);

        let name = row.take("name");
        assert_eq!(name, Some(DbValue::String(Some(Box::new("Bob".to_string())))));
        
        // After take, the field should be gone
        assert!(row.get("name").is_none());
        
        // Other fields should remain
        assert!(row.get("age").is_some());
    }

    #[test]
    fn test_get_i32_helper() {
        let mut row = DbRow::new();
        row.insert("count", 100i32);

        let val = row.get_i32("count");
        assert!(val.is_ok());
        assert_eq!(*val.unwrap(), 100);
    }

    #[test]
    fn test_get_i32_returns_error_for_null() {
        let mut row = DbRow::new();
        row.insert("nullable_int", None::<i32>);

        let val = row.get_i32("nullable_int");
        assert!(val.is_err());
    }

    #[test]
    fn test_get_i32_returns_error_for_wrong_type() {
        let mut row = DbRow::new();
        row.insert("name", "not_an_int");

        let val = row.get_i32("name");
        assert!(val.is_err());
    }

    #[test]
    fn test_get_string_helper() {
        let mut row = DbRow::new();
        row.insert("name", "Charlie");

        let val = row.get_string("name");
        assert!(val.is_ok());
        assert_eq!(*val.unwrap(), "Charlie");
    }

    #[test]
    fn test_take_i32_helper() {
        let mut row = DbRow::new();
        row.insert("score", 95i32);

        let val = row.take_i32("score");
        assert!(val.is_ok());
        assert_eq!(val.unwrap(), 95);
        
        // After take, field should be removed
        assert!(row.get("score").is_none());
    }

    #[test]
    fn test_take_string_helper() {
        let mut row = DbRow::new();
        row.insert("description", "A long description");

        let val = row.take_string("description");
        assert!(val.is_ok());
        assert_eq!(val.unwrap(), "A long description");
        
        assert!(row.get("description").is_none());
    }

    #[test]
    fn test_row_from_iter() {
        let items = vec![
            ("id".to_string(), DbValue::I64(Some(1))),
            ("name".to_string(), DbValue::String(Some(Box::new("Diana".to_string())))),
        ];

        let row: DbRow = items.into_iter().collect();
        assert_eq!(row.0.len(), 2);
        assert!(row.get("id").is_some());
        assert!(row.get("name").is_some());
    }

    #[test]
    fn test_row_into_iter() {
        let mut row = DbRow::new();
        row.insert("x", 10i32);
        row.insert("y", 20i32);

        let mut count = 0;
        for (key, val) in row {
            count += 1;
            assert!(["x", "y"].contains(&key.as_str()));
            assert!(matches!(val, DbValue::I32(Some(_))));
        }
        
        assert_eq!(count, 2);
    }

    #[test]
    fn test_get_bool_helper() {
        let mut row = DbRow::new();
        row.insert("is_admin", true);

        let val = row.get_bool("is_admin");
        assert!(val.is_ok());
        assert_eq!(*val.unwrap(), true);
    }

    #[test]
    fn test_get_uuid_helper() {
        let mut row = DbRow::new();
        let uuid = Uuid::nil();
        row.insert("id", uuid);

        let val = row.get_uuid("id");
        assert!(val.is_ok());
        assert_eq!(*val.unwrap(), uuid);
    }

    #[test]
    fn test_get_f64_helper() {
        let mut row = DbRow::new();
        row.insert("price", 19.99f64);

        let val = row.get_f64("price");
        assert!(val.is_ok());
        assert_eq!(*val.unwrap(), 19.99);
    }

    #[test]
    fn test_multiple_numeric_types() {
        let mut row = DbRow::new();
        row.insert("i8_val", 10i8);
        row.insert("i16_val", 1000i16);
        row.insert("i32_val", 100000i32);
        row.insert("i64_val", 1000000000i64);
        row.insert("u32_val", 42u32);
        row.insert("f32_val", 1.5f32);

        assert!(row.get_i8("i8_val").is_ok());
        assert!(row.get_i16("i16_val").is_ok());
        assert!(row.get_i32("i32_val").is_ok());
        assert!(row.get_i64("i64_val").is_ok());
        assert!(row.get_u32("u32_val").is_ok());
        assert!(row.get_f32("f32_val").is_ok());
    }
}