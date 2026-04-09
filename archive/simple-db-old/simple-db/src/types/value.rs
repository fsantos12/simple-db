//! Type-safe database values with support for SQL and specialized types.
//!
//! `DbValue` represents a single cell value.
//! - `DbValue::Null` represents NULL for all types (no per-variant `Option<T>`).
//! - Variants store owned values directly (no extra boxing by default).

use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use serde_json::Value as JsonValue;
use rust_decimal::Decimal;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueType {
    Null,
    I8,
    I16,
    I32,
    I64,
    I128,
    U8,
    U16,
    U32,
    U64,
    U128,
    F32,
    F64,
    Bool,
    Char,
    Date,
    Time,
    Timestamp,
    Timestamptz,
    Decimal,
    String,
    Bytes,
    Uuid,
    Json,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DbValue {
    Null,

    // Primitive types
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    F32(f32),
    F64(f64),
    Bool(bool),
    Char(char),

    // Temporal types
    Date(NaiveDate),
    Time(NaiveTime),
    Timestamp(NaiveDateTime),
    Timestamptz(DateTime<Utc>),

    // Owned types
    Decimal(Decimal),
    String(String),
    Bytes(Vec<u8>),
    Uuid(Uuid),
    Json(JsonValue),
}

impl DbValue {
    #[inline]
    pub fn is_null(&self) -> bool {
        matches!(self, DbValue::Null)
    }

    #[inline]
    pub fn value_type(&self) -> ValueType {
        match self {
            DbValue::Null => ValueType::Null,
            DbValue::I8(_) => ValueType::I8,
            DbValue::I16(_) => ValueType::I16,
            DbValue::I32(_) => ValueType::I32,
            DbValue::I64(_) => ValueType::I64,
            DbValue::I128(_) => ValueType::I128,
            DbValue::U8(_) => ValueType::U8,
            DbValue::U16(_) => ValueType::U16,
            DbValue::U32(_) => ValueType::U32,
            DbValue::U64(_) => ValueType::U64,
            DbValue::U128(_) => ValueType::U128,
            DbValue::F32(_) => ValueType::F32,
            DbValue::F64(_) => ValueType::F64,
            DbValue::Bool(_) => ValueType::Bool,
            DbValue::Char(_) => ValueType::Char,
            DbValue::Date(_) => ValueType::Date,
            DbValue::Time(_) => ValueType::Time,
            DbValue::Timestamp(_) => ValueType::Timestamp,
            DbValue::Timestamptz(_) => ValueType::Timestamptz,
            DbValue::Decimal(_) => ValueType::Decimal,
            DbValue::String(_) => ValueType::String,
            DbValue::Bytes(_) => ValueType::Bytes,
            DbValue::Uuid(_) => ValueType::Uuid,
            DbValue::Json(_) => ValueType::Json,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DbValueRef<'a> {
    Null,
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    F32(f32),
    F64(f64),
    Bool(bool),
    Char(char),
    Date(&'a NaiveDate),
    Time(&'a NaiveTime),
    Timestamp(&'a NaiveDateTime),
    Timestamptz(&'a DateTime<Utc>),
    Decimal(&'a Decimal),
    String(&'a str),
    Bytes(&'a [u8]),
    Uuid(&'a Uuid),
    Json(&'a JsonValue),
}

impl<'a> From<&'a DbValue> for DbValueRef<'a> {
    fn from(v: &'a DbValue) -> Self {
        match v {
            DbValue::Null => DbValueRef::Null,
            DbValue::I8(x) => DbValueRef::I8(*x),
            DbValue::I16(x) => DbValueRef::I16(*x),
            DbValue::I32(x) => DbValueRef::I32(*x),
            DbValue::I64(x) => DbValueRef::I64(*x),
            DbValue::I128(x) => DbValueRef::I128(*x),
            DbValue::U8(x) => DbValueRef::U8(*x),
            DbValue::U16(x) => DbValueRef::U16(*x),
            DbValue::U32(x) => DbValueRef::U32(*x),
            DbValue::U64(x) => DbValueRef::U64(*x),
            DbValue::U128(x) => DbValueRef::U128(*x),
            DbValue::F32(x) => DbValueRef::F32(*x),
            DbValue::F64(x) => DbValueRef::F64(*x),
            DbValue::Bool(x) => DbValueRef::Bool(*x),
            DbValue::Char(x) => DbValueRef::Char(*x),
            DbValue::Date(x) => DbValueRef::Date(x),
            DbValue::Time(x) => DbValueRef::Time(x),
            DbValue::Timestamp(x) => DbValueRef::Timestamp(x),
            DbValue::Timestamptz(x) => DbValueRef::Timestamptz(x),
            DbValue::Decimal(x) => DbValueRef::Decimal(x),
            DbValue::String(x) => DbValueRef::String(x.as_str()),
            DbValue::Bytes(x) => DbValueRef::Bytes(x.as_slice()),
            DbValue::Uuid(x) => DbValueRef::Uuid(x),
            DbValue::Json(x) => DbValueRef::Json(x),
        }
    }
}

macro_rules! impl_from_owned {
    ($t:ty, $variant:ident) => {
        impl From<$t> for DbValue {
            fn from(v: $t) -> Self {
                DbValue::$variant(v)
            }
        }
    };
}

impl_from_owned!(i8, I8);
impl_from_owned!(i16, I16);
impl_from_owned!(i32, I32);
impl_from_owned!(i64, I64);
impl_from_owned!(i128, I128);
impl_from_owned!(u8, U8);
impl_from_owned!(u16, U16);
impl_from_owned!(u32, U32);
impl_from_owned!(u64, U64);
impl_from_owned!(u128, U128);
impl_from_owned!(f32, F32);
impl_from_owned!(f64, F64);
impl_from_owned!(bool, Bool);
impl_from_owned!(char, Char);
impl_from_owned!(NaiveDate, Date);
impl_from_owned!(NaiveTime, Time);
impl_from_owned!(NaiveDateTime, Timestamp);
impl_from_owned!(DateTime<Utc>, Timestamptz);
impl_from_owned!(Decimal, Decimal);
impl_from_owned!(String, String);
impl_from_owned!(Vec<u8>, Bytes);
impl_from_owned!(Uuid, Uuid);
impl_from_owned!(JsonValue, Json);

impl From<&str> for DbValue {
    fn from(val: &str) -> Self {
        DbValue::String(val.to_string())
    }
}

impl<T> From<Option<T>> for DbValue
where
    DbValue: From<T>,
{
    fn from(val: Option<T>) -> Self {
        match val {
            Some(v) => v.into(),
            None => DbValue::Null,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_null_with_some_values() {
        assert!(!DbValue::I32(42).is_null());
        assert!(!DbValue::String("hello".to_string()).is_null());
        assert!(!DbValue::Bool(true).is_null());
    }

    #[test]
    fn test_is_null_with_none_values() {
        assert!(DbValue::Null.is_null());
    }

    #[test]
    fn test_conversion_from_primitive_types() {
        let val: DbValue = 42i32.into();
        assert_eq!(val, DbValue::I32(42));

        let val: DbValue = true.into();
        assert_eq!(val, DbValue::Bool(true));

        let val: DbValue = 3.14f64.into();
        assert_eq!(val, DbValue::F64(3.14));
    }

    #[test]
    fn test_conversion_from_string() {
        let val: DbValue = "hello".into();
        assert_eq!(val, DbValue::String("hello".to_string()));

        let val: DbValue = String::from("world").into();
        assert_eq!(val, DbValue::String("world".to_string()));
    }

    #[test]
    fn test_conversion_from_option() {
        let val: DbValue = Some(42i32).into();
        assert_eq!(val, DbValue::I32(42));

        let val: DbValue = None::<i32>.into();
        assert_eq!(val, DbValue::Null);
    }

    #[test]
    fn test_conversion_from_option_string() {
        let val: DbValue = Some("hello").into();
        assert_eq!(val, DbValue::String("hello".to_string()));

        let val: DbValue = None::<&str>.into();
        assert_eq!(val, DbValue::Null);
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