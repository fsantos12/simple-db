use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use serde_json::Value as JsonValue;
use rust_decimal::Decimal;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub enum DbValue {
    I8(Option<i8>), I16(Option<i16>), I32(Option<i32>), I64(Option<i64>), I128(Option<i128>),
    U8(Option<u8>), U16(Option<u16>), U32(Option<u32>), U64(Option<u64>), U128(Option<u128>),
    F32(Option<f32>), F64(Option<f64>), Decimal(Option<Decimal>),
    Bool(Option<bool>),
    Char(Option<char>), String(Option<String>),
    Date(Option<NaiveDate>), Time(Option<NaiveTime>), Timestamp(Option<NaiveDateTime>), Timestamptz(Option<DateTime<Utc>>),
    Bytes(Option<Vec<u8>>),
    Uuid(Option<Uuid>),
    Json(Option<JsonValue>)
}

impl DbValue {
    pub fn is_null(&self) -> bool {
        match self {
            DbValue::I8(v) => v.is_none(),
            DbValue::I16(v) => v.is_none(),
            DbValue::I32(v) => v.is_none(),
            DbValue::I64(v) => v.is_none(),
            DbValue::I128(v) => v.is_none(),
            DbValue::U8(v) => v.is_none(),
            DbValue::U16(v) => v.is_none(),
            DbValue::U32(v) => v.is_none(),
            DbValue::U64(v) => v.is_none(),
            DbValue::U128(v) => v.is_none(),
            DbValue::F32(v) => v.is_none(),
            DbValue::F64(v) => v.is_none(),
            DbValue::Decimal(v) => v.is_none(),
            DbValue::Bool(v) => v.is_none(),
            DbValue::Char(v) => v.is_none(),
            DbValue::String(v) => v.is_none(),
            DbValue::Date(v) => v.is_none(),
            DbValue::Time(v) => v.is_none(),
            DbValue::Timestamp(v) => v.is_none(),
            DbValue::Timestamptz(v) => v.is_none(),
            DbValue::Bytes(v) => v.is_none(),
            DbValue::Uuid(v) => v.is_none(),
            DbValue::Json(v) => v.is_none()
        }
    }
}

macro_rules! impl_from_value {
    ($variant:ident, $type:ty) => {
        impl From<$type> for DbValue {
            fn from(val: $type) -> Self {
                DbValue::$variant(Some(val))
            }
        }
        impl From<Option<$type>> for DbValue {
            fn from(val: Option<$type>) -> Self {
                DbValue::$variant(val)
            }
        }
    };
}

impl_from_value!(I8, i8);
impl_from_value!(I16, i16);
impl_from_value!(I32, i32);
impl_from_value!(I64, i64);
impl_from_value!(I128, i128);
impl_from_value!(U8, u8);
impl_from_value!(U16, u16);
impl_from_value!(U32, u32);
impl_from_value!(U64, u64);
impl_from_value!(U128, u128);
impl_from_value!(F32, f32);
impl_from_value!(F64, f64);
impl_from_value!(Decimal, Decimal);
impl_from_value!(Bool, bool);
impl_from_value!(Char, char);
impl_from_value!(String, String);
impl_from_value!(Date, NaiveDate);
impl_from_value!(Time, NaiveTime);
impl_from_value!(Timestamp, NaiveDateTime);
impl_from_value!(Timestamptz, DateTime<Utc>);
impl_from_value!(Bytes, Vec<u8>);
impl_from_value!(Uuid, Uuid);
impl_from_value!(Json, JsonValue);

impl From<&str> for DbValue {
    fn from(val: &str) -> Self {
        DbValue::String(Some(val.to_string()))
    }
}

impl From<Option<&str>> for DbValue {
    fn from(val: Option<&str>) -> Self {
        DbValue::String(val.map(|s| s.to_string()))
    }
}