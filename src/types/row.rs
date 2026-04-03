use std::collections::HashMap;

use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use serde_json::Value as JsonValue;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::types::value::DbValue;

#[derive(Debug, Clone, Default)]
pub struct DbRow(pub HashMap<String, DbValue>);

macro_rules! impl_type_helpers {
    ($suffix:ident, $variant:ident, $type:ty) => {
        paste::paste! {
            // --- 1. get_x (Strict Reference) ---
            pub fn [<get_ $suffix>](&self, key: &str) -> Result<&$type, String> {
                match self.get(key) {
                    Some(DbValue::$variant(Some(v))) => Ok(v),
                    Some(DbValue::$variant(None)) => Err(format!("Field '{}' is NULL", key)),
                    Some(_) => Err(format!("Field '{}' is not a {}", key, stringify!($suffix))),
                    None => Err(format!("Field '{}' is missing", key)),
                }
            }

            // --- 2. get_opt_x (Optional Reference) ---
            pub fn [<get_opt_ $suffix>](&self, key: &str) -> Option<&$type> {
                match self.get(key) {
                    Some(DbValue::$variant(Some(v))) => Some(v),
                    _ => None,
                }
            }

            // --- 3. take_x (Strict Owned) ---
            pub fn [<take_ $suffix>](&mut self, key: &str) -> Result<$type, String> {
                match self.take(key) {
                    Some(DbValue::$variant(Some(v))) => Ok(v),
                    Some(DbValue::$variant(None)) => Err(format!("Field '{}' is NULL", key)),
                    Some(_) => Err(format!("Field '{}' is not a {}", key, stringify!($suffix))),
                    None => Err(format!("Field '{}' is missing", key)),
                }
            }

            // --- 4. take_opt_x (Optional Owned) ---
            pub fn [<take_opt_ $suffix>](&mut self, key: &str) -> Option<$type> {
                match self.take(key) {
                    Some(DbValue::$variant(Some(v))) => Some(v),
                    _ => None,
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
    impl_type_helpers!(decimal, Decimal, Decimal);
    impl_type_helpers!(bool, Bool, bool);
    impl_type_helpers!(char, Char, char);
    impl_type_helpers!(string, String, String);
    impl_type_helpers!(date, Date, NaiveDate);
    impl_type_helpers!(time, Time, NaiveTime);
    impl_type_helpers!(timestamp, Timestamp, NaiveDateTime);
    impl_type_helpers!(timestamptz, Timestamptz, DateTime<Utc>);
    impl_type_helpers!(bytes, Bytes, Vec<u8>);
    impl_type_helpers!(uuid, Uuid, Uuid);
    impl_type_helpers!(json, Json, JsonValue);
}

// This allows: .collect::<DbRow>()
impl FromIterator<(String, DbValue)> for DbRow {
    fn from_iter<I: IntoIterator<Item = (String, DbValue)>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}
