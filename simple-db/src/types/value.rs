//! Type-safe database values with support for SQL and specialized types.
//!
//! `DbValue` is an enum wrapper supporting primitives (integers, floats, booleans),
//! temporal types (Date, Time, Timestamp), and specialized types (UUID, Decimal,
//! JSON). String values and large types are boxed for memory efficiency.

use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use serde_json::Value as JsonValue;
use rust_decimal::Decimal;
use uuid::Uuid;

/// Helper macro to generate the type definition inside the variant.
/// It wraps the type in Box if the 'boxed' keyword is provided.
macro_rules! wrap_type {
    ($type:ty, boxed) => { Box<$type> };
    ($type:ty,) => { $type };
}

/// Helper macro to wrap the value during conversion.
macro_rules! wrap_val {
    ($val:expr, boxed) => { Box::new($val) };
    ($val:expr,) => { $val };
}

macro_rules! define_db_value {
    ($( $variant:ident($type:ty $(, $boxed:ident)?) ),* $(,)?) => {
        #[derive(Debug, Clone, PartialEq)]
        pub enum DbValue {
            $(
                $variant(Option<wrap_type!($type, $($boxed)?)>),
            )*
        }

        impl DbValue {
            /// Returns true if the inner value is None, regardless of the variant.
            pub fn is_null(&self) -> bool {
                match self {
                    $( DbValue::$variant(v) => v.is_none(), )*
                }
            }
        }

        $(
            // implementation of From<T> for DbValue
            impl From<$type> for DbValue {
                fn from(val: $type) -> Self {
                    DbValue::$variant(Some(wrap_val!(val, $($boxed)?)))
                }
            }

            // implementation of From<Option<T>> for DbValue
            impl From<Option<$type>> for DbValue {
                fn from(val: Option<$type>) -> Self {
                    DbValue::$variant(val.map(|v| wrap_val!(v, $($boxed)?)))
                }
            }
        )*
    };
}

// --- FULL IMPLEMENTATION ---
define_db_value! {
    // Primitive types
    I8(i8), I16(i16), I32(i32), I64(i64), I128(i128),
    U8(u8), U16(u16), U32(u32), U64(u64), U128(u128),
    F32(f32), F64(f64), 
    Bool(bool),
    Char(char),

    // Temporal types
    Date(NaiveDate), 
    Time(NaiveTime), 
    Timestamp(NaiveDateTime), 
    Timestamptz(DateTime<Utc>),

    // Large types marked as 'boxed' for memory efficiency [1, 2]
    Decimal(Decimal, boxed),
    String(String, boxed),
    Bytes(Vec<u8>, boxed),
    Uuid(Uuid, boxed),
    Json(JsonValue, boxed),
}

/// Manual implementations for string slices (&str) to improve ergonomics.
/// We don't include this in the macro to avoid conflicting with the String variant.
impl From<&str> for DbValue {
    fn from(val: &str) -> Self {
        DbValue::String(Some(Box::new(val.to_string())))
    }
}

impl From<Option<&str>> for DbValue {
    fn from(val: Option<&str>) -> Self {
        DbValue::String(val.map(|s| Box::new(s.to_string())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_null_with_some_values() {
        assert!(!DbValue::I32(Some(42)).is_null());
        assert!(!DbValue::String(Some(Box::new("hello".to_string()))).is_null());
        assert!(!DbValue::Bool(Some(true)).is_null());
    }

    #[test]
    fn test_is_null_with_none_values() {
        assert!(DbValue::I32(None).is_null());
        assert!(DbValue::String(None).is_null());
        assert!(DbValue::Bool(None).is_null());
    }

    #[test]
    fn test_conversion_from_primitive_types() {
        let val: DbValue = 42i32.into();
        assert_eq!(val, DbValue::I32(Some(42)));

        let val: DbValue = true.into();
        assert_eq!(val, DbValue::Bool(Some(true)));

        let val: DbValue = 3.14f64.into();
        assert_eq!(val, DbValue::F64(Some(3.14)));
    }

    #[test]
    fn test_conversion_from_string() {
        let val: DbValue = "hello".into();
        assert_eq!(val, DbValue::String(Some(Box::new("hello".to_string()))));

        let val: DbValue = String::from("world").into();
        assert_eq!(val, DbValue::String(Some(Box::new("world".to_string()))));
    }

    #[test]
    fn test_conversion_from_option() {
        let val: DbValue = Some(42i32).into();
        assert_eq!(val, DbValue::I32(Some(42)));

        let val: DbValue = None::<i32>.into();
        assert_eq!(val, DbValue::I32(None));
    }

    #[test]
    fn test_conversion_from_option_string() {
        let val: DbValue = Some("hello").into();
        assert_eq!(val, DbValue::String(Some(Box::new("hello".to_string()))));

        let val: DbValue = None::<&str>.into();
        assert_eq!(val, DbValue::String(None));
    }

    #[test]
    fn test_uuid_conversion() {
        let uuid = Uuid::nil();
        let val: DbValue = uuid.into();
        assert!(!val.is_null());
        
        let val: DbValue = Some(uuid).into();
        assert!(!val.is_null());
    }

    #[test]
    fn test_decimal_conversion() {
        let dec = Decimal::from(123);
        let val: DbValue = dec.into();
        assert!(!val.is_null());
    }

    #[test]
    fn test_datetime_conversion() {
        let now = Utc::now();
        let val: DbValue = now.into();
        assert!(!val.is_null());
    }

    #[test]
    fn test_json_conversion() {
        let json = serde_json::json!({"key": "value"});
        let val: DbValue = json.into();
        assert!(!val.is_null());
    }

    #[test]
    fn test_bytes_conversion() {
        let bytes = vec![1, 2, 3, 4, 5];
        let val: DbValue = bytes.into();
        assert!(!val.is_null());
    }
}