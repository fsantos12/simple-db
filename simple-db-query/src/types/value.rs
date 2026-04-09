use std::ptr::NonNull;

use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use serde_json::Value as JsonValue;
use rust_decimal::Decimal;
use uuid::Uuid;

// Tag in the high 16 bits, payload in the low 48.
const TAG_SHIFT: u64    = 48;
const PAYLOAD_MASK: u64 = (1 << TAG_SHIFT) - 1;

// To extract Category and Type from the 16-bit tag
const CATEGORY_MASK: u64 = 0b1100000000000000;
const TYPE_MASK: u64     = 0b0011111111111111;

// Category encodes how the payload should be interpreted.
// - INLINE: payload is value bits (no allocation)
// - BOXED:  payload is a pointer to a heap allocation (Drop must free)
const CATEGORY_INLINE: u64 = 0b0000000000000000;
const CATEGORY_BOXED: u64  = 0b0100000000000000;

// Value types.
const TYPE_NULL: u64        = 0;
const TYPE_BOOL: u64        = 1;
const TYPE_I8: u64          = 2;
const TYPE_I16: u64         = 3;
const TYPE_I32: u64         = 4;
const TYPE_I64: u64         = 5;
const TYPE_I128: u64        = 6;
const TYPE_U8: u64          = 7;
const TYPE_U16: u64         = 8;
const TYPE_U32: u64         = 9;
const TYPE_U64: u64         = 10;
const TYPE_U128: u64        = 11;
const TYPE_F32: u64         = 12;
const TYPE_F64: u64         = 13;
const TYPE_DECIMAL: u64     = 14;
const TYPE_CHAR: u64        = 15;
const TYPE_STRING: u64      = 16;
const TYPE_DATE: u64        = 17;
const TYPE_TIME: u64        = 18;
const TYPE_TIMESTAMP: u64   = 19;
const TYPE_TIMESTAMPZ: u64  = 20;
const TYPE_BYTES: u64       = 21;
const TYPE_UUID: u64        = 22;
const TYPE_JSON: u64        = 23;

/// Compact DB value: `(tag << 48) | payload`.
///
/// - **Inline values** store their bits directly in the 48-bit payload (no allocation).
/// - **Heap values** store a pointer in the 48-bit payload (allocation + `Drop`).
pub struct DbValue(u64);

impl DbValue {
    #[inline]
    fn mk_tag(category: u64, ty: u64) -> u64 {
        debug_assert_eq!(category & !CATEGORY_MASK, 0);
        debug_assert_eq!(ty & !TYPE_MASK, 0);
        category | ty
    }

    #[inline]
    fn from_tag_and_payload(tag: u64, payload: u64) -> Self {
        debug_assert!(tag < (1 << 16));
        debug_assert_eq!(payload & !PAYLOAD_MASK, 0);
        Self((tag << TAG_SHIFT) | payload)
    }

    #[inline]
    fn from_tag_and_i48(tag: u64, val: i64) -> Self {
        // Store in 2's complement within 48 bits.
        let payload = (val as u64) & PAYLOAD_MASK;
        Self::from_tag_and_payload(tag, payload)
    }

    #[inline]
    fn from_tag_and_u48(tag: u64, val: u64) -> Self {
        debug_assert_eq!(val & !PAYLOAD_MASK, 0, "value does not fit in 48 bits");
        Self::from_tag_and_payload(tag, val & PAYLOAD_MASK)
    }

    #[inline]
    fn from_tag_and_boxed<T>(tag: u64, val: T) -> Self {
        // Assumes the target uses 48-bit (or less) canonical user-space addresses.
        let raw = Box::into_raw(Box::new(val));
        let ptr = NonNull::new(raw).expect("Box::into_raw returned null");
        let addr = ptr.as_ptr() as usize as u64;
        debug_assert_eq!(addr & !PAYLOAD_MASK, 0, "pointer does not fit in 48 bits");
        Self::from_tag_and_payload(tag, addr & PAYLOAD_MASK)
    }

    #[inline]
    fn tag(&self) -> u64 {
        self.0 >> TAG_SHIFT
    }

    #[inline]
    fn category(&self) -> u64 {
        self.tag() & CATEGORY_MASK
    }

    #[inline]
    fn ty(&self) -> u64 {
        self.tag() & TYPE_MASK
    }

    #[inline]
    fn payload(&self) -> u64 {
        self.0 & PAYLOAD_MASK
    }

    #[inline]
    fn payload_as_i64_i48(&self) -> i64 {
        // Sign-extend from 48-bit 2's complement.
        let p = self.payload();
        let sign_bit = 1u64 << 47;
        if (p & sign_bit) != 0 {
            (p | !PAYLOAD_MASK) as i64
        } else {
            p as i64
        }
    }

    #[inline]
    unsafe fn payload_as_ref<T>(&self) -> &T {
        unsafe { &*(self.payload() as usize as *const T) }
    }

    // ---------------------------------------------------------------------
    // Heap values (allocation): listed first on purpose.
    // ---------------------------------------------------------------------

    #[inline]
    pub fn from_i64(val: i64) -> Self {
        // i64 can be inline (i48) or boxed, depending on range.
        const MIN_I48: i64 = -(1i64 << 47);
        const MAX_I48: i64 = (1i64 << 47) - 1;
        if (MIN_I48..=MAX_I48).contains(&val) {
            Self::from_tag_and_i48(Self::mk_tag(CATEGORY_INLINE, TYPE_I64), val)
        } else {
            Self::from_tag_and_boxed(Self::mk_tag(CATEGORY_BOXED, TYPE_I64), val)
        }
    }

    #[inline]
    pub fn as_i64(&self) -> Option<i64> {
        if self.ty() != TYPE_I64 {
            return None;
        }
        match self.category() {
            CATEGORY_INLINE => Some(self.payload_as_i64_i48()),
            CATEGORY_BOXED => Some(unsafe { *self.payload_as_ref::<i64>() }),
            _ => None,
        }
    }

    #[inline]
    pub fn from_u64(val: u64) -> Self {
        // u64 can be inline (u48) or boxed, depending on range.
        if (val & !PAYLOAD_MASK) == 0 {
            Self::from_tag_and_u48(Self::mk_tag(CATEGORY_INLINE, TYPE_U64), val)
        } else {
            Self::from_tag_and_boxed(Self::mk_tag(CATEGORY_BOXED, TYPE_U64), val)
        }
    }

    #[inline]
    pub fn as_u64(&self) -> Option<u64> {
        if self.ty() != TYPE_U64 { return None; }
        match self.category() {
            CATEGORY_INLINE => Some(self.payload()),
            CATEGORY_BOXED => Some(unsafe { *self.payload_as_ref::<u64>() }),
            _ => None,
        }
    }

    #[inline]
    pub fn from_f64(val: f64) -> Self {
        // f64 always boxes in this layout (it doesn't fit in 48 bits).
        Self::from_tag_and_boxed(Self::mk_tag(CATEGORY_BOXED, TYPE_F64), val.to_bits())
    }

    #[inline]
    pub fn as_f64(&self) -> Option<f64> {
        if self.ty() != TYPE_F64 || self.category() != CATEGORY_BOXED {
            return None;
        }
        let bits = unsafe { *self.payload_as_ref::<u64>() };
        Some(f64::from_bits(bits))
    }

    #[inline]
    pub fn from_i128(val: i128) -> Self {
        Self::from_tag_and_boxed(Self::mk_tag(CATEGORY_BOXED, TYPE_I128), val)
    }

    #[inline]
    pub fn as_i128(&self) -> Option<&i128> {
        (self.ty() == TYPE_I128 && self.category() == CATEGORY_BOXED)
            .then(|| unsafe { self.payload_as_ref::<i128>() })
    }

    #[inline]
    pub fn from_u128(val: u128) -> Self {
        Self::from_tag_and_boxed(Self::mk_tag(CATEGORY_BOXED, TYPE_U128), val)
    }

    #[inline]
    pub fn as_u128(&self) -> Option<&u128> {
        (self.ty() == TYPE_U128 && self.category() == CATEGORY_BOXED)
            .then(|| unsafe { self.payload_as_ref::<u128>() })
    }

    #[inline]
    pub fn from_decimal(val: Decimal) -> Self {
        Self::from_tag_and_boxed(Self::mk_tag(CATEGORY_BOXED, TYPE_DECIMAL), val)
    }

    #[inline]
    pub fn as_decimal(&self) -> Option<&Decimal> {
        (self.ty() == TYPE_DECIMAL && self.category() == CATEGORY_BOXED)
            .then(|| unsafe { self.payload_as_ref::<Decimal>() })
    }

    #[inline]
    pub fn from_string(val: impl Into<String>) -> Self {
        Self::from_tag_and_boxed(Self::mk_tag(CATEGORY_BOXED, TYPE_STRING), val.into())
    }

    #[inline]
    pub fn as_string(&self) -> Option<&str> {
        if self.ty() != TYPE_STRING || self.category() != CATEGORY_BOXED {
            return None;
        }
        Some(unsafe { self.payload_as_ref::<String>().as_str() })
    }

    #[inline]
    pub fn from_bytes(val: impl Into<Vec<u8>>) -> Self {
        Self::from_tag_and_boxed(Self::mk_tag(CATEGORY_BOXED, TYPE_BYTES), val.into())
    }

    #[inline]
    pub fn as_bytes(&self) -> Option<&[u8]> {
        if self.ty() != TYPE_BYTES || self.category() != CATEGORY_BOXED {
            return None;
        }
        Some(unsafe { self.payload_as_ref::<Vec<u8>>().as_slice() })
    }

    #[inline]
    pub fn from_uuid(val: Uuid) -> Self {
        Self::from_tag_and_boxed(Self::mk_tag(CATEGORY_BOXED, TYPE_UUID), val)
    }

    #[inline]
    pub fn as_uuid(&self) -> Option<&Uuid> {
        (self.ty() == TYPE_UUID && self.category() == CATEGORY_BOXED)
            .then(|| unsafe { self.payload_as_ref::<Uuid>() })
    }

    #[inline]
    pub fn from_json(val: impl Into<JsonValue>) -> Self {
        Self::from_tag_and_boxed(Self::mk_tag(CATEGORY_BOXED, TYPE_JSON), val.into())
    }

    #[inline]
    pub fn as_json(&self) -> Option<&JsonValue> {
        (self.ty() == TYPE_JSON && self.category() == CATEGORY_BOXED)
            .then(|| unsafe { self.payload_as_ref::<JsonValue>() })
    }

    #[inline]
    pub fn from_date(val: NaiveDate) -> Self {
        Self::from_tag_and_boxed(Self::mk_tag(CATEGORY_BOXED, TYPE_DATE), val)
    }

    #[inline]
    pub fn as_date(&self) -> Option<&NaiveDate> {
        (self.ty() == TYPE_DATE && self.category() == CATEGORY_BOXED)
            .then(|| unsafe { self.payload_as_ref::<NaiveDate>() })
    }

    #[inline]
    pub fn from_time(val: NaiveTime) -> Self {
        Self::from_tag_and_boxed(Self::mk_tag(CATEGORY_BOXED, TYPE_TIME), val)
    }

    #[inline]
    pub fn as_time(&self) -> Option<&NaiveTime> {
        (self.ty() == TYPE_TIME && self.category() == CATEGORY_BOXED)
            .then(|| unsafe { self.payload_as_ref::<NaiveTime>() })
    }

    #[inline]
    pub fn from_timestamp(val: NaiveDateTime) -> Self {
        Self::from_tag_and_boxed(Self::mk_tag(CATEGORY_BOXED, TYPE_TIMESTAMP), val)
    }

    #[inline]
    pub fn as_timestamp(&self) -> Option<&NaiveDateTime> {
        (self.ty() == TYPE_TIMESTAMP && self.category() == CATEGORY_BOXED)
            .then(|| unsafe { self.payload_as_ref::<NaiveDateTime>() })
    }

    #[inline]
    pub fn from_timestampz(val: DateTime<Utc>) -> Self {
        Self::from_tag_and_boxed(Self::mk_tag(CATEGORY_BOXED, TYPE_TIMESTAMPZ), val)
    }

    #[inline]
    pub fn as_timestampz(&self) -> Option<&DateTime<Utc>> {
        (self.ty() == TYPE_TIMESTAMPZ && self.category() == CATEGORY_BOXED)
            .then(|| unsafe { self.payload_as_ref::<DateTime<Utc>>() })
    }

    // ---------------------------------------------------------------------
    // Inline values (no allocation): listed after heap values on purpose.
    // ---------------------------------------------------------------------

    #[inline]
    pub fn from_null() -> Self {
        Self::from_tag_and_payload(Self::mk_tag(CATEGORY_INLINE, TYPE_NULL), 0)
    }

    #[inline]
    pub fn is_null(&self) -> bool {
        self.ty() == TYPE_NULL
    }

    #[inline]
    pub fn from_bool(val: bool) -> Self {
        Self::from_tag_and_u48(Self::mk_tag(CATEGORY_INLINE, TYPE_BOOL), if val { 1 } else { 0 })
    }

    #[inline]
    pub fn as_bool(&self) -> Option<bool> {
        (self.ty() == TYPE_BOOL).then(|| self.payload() != 0)
    }

    #[inline]
    pub fn from_i8(val: i8) -> Self {
        Self::from_tag_and_i48(Self::mk_tag(CATEGORY_INLINE, TYPE_I8), val as i64)
    }

    #[inline]
    pub fn as_i8(&self) -> Option<i8> {
        (self.ty() == TYPE_I8).then(|| self.payload_as_i64_i48() as i8)
    }

    #[inline]
    pub fn from_i16(val: i16) -> Self {
        Self::from_tag_and_i48(Self::mk_tag(CATEGORY_INLINE, TYPE_I16), val as i64)
    }

    #[inline]
    pub fn as_i16(&self) -> Option<i16> {
        (self.ty() == TYPE_I16).then(|| self.payload_as_i64_i48() as i16)
    }

    #[inline]
    pub fn from_i32(val: i32) -> Self {
        Self::from_tag_and_i48(Self::mk_tag(CATEGORY_INLINE, TYPE_I32), val as i64)
    }

    #[inline]
    pub fn as_i32(&self) -> Option<i32> {
        (self.ty() == TYPE_I32).then(|| self.payload_as_i64_i48() as i32)
    }

    #[inline]
    pub fn from_u8(val: u8) -> Self {
        Self::from_tag_and_u48(Self::mk_tag(CATEGORY_INLINE, TYPE_U8), val as u64)
    }

    #[inline]
    pub fn as_u8(&self) -> Option<u8> {
        (self.ty() == TYPE_U8).then(|| self.payload() as u8)
    }

    #[inline]
    pub fn from_u16(val: u16) -> Self {
        Self::from_tag_and_u48(Self::mk_tag(CATEGORY_INLINE, TYPE_U16), val as u64)
    }

    #[inline]
    pub fn as_u16(&self) -> Option<u16> {
        (self.ty() == TYPE_U16).then(|| self.payload() as u16)
    }

    #[inline]
    pub fn from_u32(val: u32) -> Self {
        Self::from_tag_and_u48(Self::mk_tag(CATEGORY_INLINE, TYPE_U32), val as u64)
    }

    #[inline]
    pub fn as_u32(&self) -> Option<u32> {
        (self.ty() == TYPE_U32).then(|| self.payload() as u32)
    }

    #[inline]
    pub fn from_f32(val: f32) -> Self {
        Self::from_tag_and_u48(Self::mk_tag(CATEGORY_INLINE, TYPE_F32), val.to_bits() as u64)
    }

    #[inline]
    pub fn as_f32(&self) -> Option<f32> {
        (self.ty() == TYPE_F32).then(|| f32::from_bits(self.payload() as u32))
    }

    #[inline]
    pub fn from_char(val: char) -> Self {
        Self::from_tag_and_u48(Self::mk_tag(CATEGORY_INLINE, TYPE_CHAR), val as u32 as u64)
    }

    #[inline]
    pub fn as_char(&self) -> Option<char> {
        if self.ty() != TYPE_CHAR {
            return None;
        }
        char::from_u32(self.payload() as u32)
    }
}

impl Drop for DbValue {
    fn drop(&mut self) {
        if self.category() != CATEGORY_BOXED { return; }
        match self.ty() {
            TYPE_I64 => unsafe { drop(Box::from_raw(self.payload() as usize as *mut i64)) },
            TYPE_I128 => unsafe { drop(Box::from_raw(self.payload() as usize as *mut i128)) },
            TYPE_U64 => unsafe { drop(Box::from_raw(self.payload() as usize as *mut u64)) },
            TYPE_U128 => unsafe { drop(Box::from_raw(self.payload() as usize as *mut u128)) },
            TYPE_F64 => unsafe { drop(Box::from_raw(self.payload() as usize as *mut u64)) },
            TYPE_DECIMAL => unsafe { drop(Box::from_raw(self.payload() as usize as *mut Decimal)) },
            TYPE_STRING => unsafe { drop(Box::from_raw(self.payload() as usize as *mut String)) },
            TYPE_DATE => unsafe { drop(Box::from_raw(self.payload() as usize as *mut NaiveDate)) },
            TYPE_TIME => unsafe { drop(Box::from_raw(self.payload() as usize as *mut NaiveTime)) },
            TYPE_TIMESTAMP => unsafe { drop(Box::from_raw(self.payload() as usize as *mut NaiveDateTime)) },
            TYPE_TIMESTAMPZ => unsafe { drop(Box::from_raw(self.payload() as usize as *mut DateTime<Utc>)) },
            TYPE_BYTES => unsafe { drop(Box::from_raw(self.payload() as usize as *mut Vec<u8>)) },
            TYPE_UUID => unsafe { drop(Box::from_raw(self.payload() as usize as *mut Uuid)) },
            TYPE_JSON => unsafe { drop(Box::from_raw(self.payload() as usize as *mut JsonValue)) },
            _ => {}
        }
    }
}

impl Clone for DbValue {
    fn clone(&self) -> Self {
        if self.category() != CATEGORY_BOXED { return Self(self.0); }
        match self.ty() {
            TYPE_I64 => DbValue::from_i64(unsafe { *self.payload_as_ref::<i64>() }),
            TYPE_U64 => DbValue::from_u64(unsafe { *self.payload_as_ref::<u64>() }),
            TYPE_F64 => DbValue::from_f64(f64::from_bits(unsafe { *self.payload_as_ref::<u64>() })),
            TYPE_I128 => DbValue::from_tag_and_boxed(Self::mk_tag(CATEGORY_BOXED, TYPE_I128), unsafe { *self.payload_as_ref::<i128>() } ),
            TYPE_U128 => DbValue::from_tag_and_boxed(Self::mk_tag(CATEGORY_BOXED, TYPE_U128), unsafe { *self.payload_as_ref::<u128>() } ),
            TYPE_DECIMAL => DbValue::from_decimal(unsafe { self.payload_as_ref::<Decimal>() }.clone()),
            TYPE_STRING => DbValue::from_string(unsafe { self.payload_as_ref::<String>() }.clone()),
            TYPE_DATE => DbValue::from_date(unsafe { *self.payload_as_ref::<NaiveDate>() }),
            TYPE_TIME => DbValue::from_time(unsafe { *self.payload_as_ref::<NaiveTime>() }),
            TYPE_TIMESTAMP => DbValue::from_timestamp(unsafe { *self.payload_as_ref::<NaiveDateTime>() }),
            TYPE_TIMESTAMPZ => DbValue::from_timestampz(unsafe { self.payload_as_ref::<DateTime<Utc>>() }.clone()),
            TYPE_BYTES => DbValue::from_bytes(unsafe { self.payload_as_ref::<Vec<u8>>() }.clone()),
            TYPE_UUID => DbValue::from_uuid(unsafe { *self.payload_as_ref::<Uuid>() }),
            TYPE_JSON => DbValue::from_json(unsafe { self.payload_as_ref::<JsonValue>() }.clone()),
            _ => Self(self.0),
        }
    }
}

impl PartialEq for DbValue {
    fn eq(&self, other: &Self) -> bool {
        if self.ty() != other.ty() { return false; }
        match self.ty() {
            TYPE_NULL => true,
            TYPE_BOOL => self.as_bool() == other.as_bool(),
            TYPE_I8 => self.as_i8() == other.as_i8(),
            TYPE_I16 => self.as_i16() == other.as_i16(),
            TYPE_I32 => self.as_i32() == other.as_i32(),
            TYPE_I64 => self.as_i64() == other.as_i64(),
            TYPE_I128 => self.as_i128() == other.as_i128(),
            TYPE_U8 => self.as_u8() == other.as_u8(),
            TYPE_U16 => self.as_u16() == other.as_u16(),
            TYPE_U32 => self.as_u32() == other.as_u32(),
            TYPE_U64 => self.as_u64() == other.as_u64(),
            TYPE_U128 => self.as_u128() == other.as_u128(),
            TYPE_F32 => self.as_f32() == other.as_f32(),
            TYPE_F64 => self.as_f64() == other.as_f64(),
            TYPE_DECIMAL => self.as_decimal() == other.as_decimal(),
            TYPE_CHAR => self.as_char() == other.as_char(),
            TYPE_STRING => self.as_string() == other.as_string(),
            TYPE_DATE => self.as_date() == other.as_date(),
            TYPE_TIME => self.as_time() == other.as_time(),
            TYPE_TIMESTAMP => self.as_timestamp() == other.as_timestamp(),
            TYPE_TIMESTAMPZ => self.as_timestampz() == other.as_timestampz(),
            TYPE_BYTES => self.as_bytes() == other.as_bytes(),
            TYPE_UUID => self.as_uuid() == other.as_uuid(),
            TYPE_JSON => self.as_json() == other.as_json(),
            _ => false,
        }
    }
}

// Implementing From<T>
macro_rules! impl_from_t {
    ($t:ty, $constructor:ident) => {
        impl From<$t> for DbValue {
            #[inline]
            fn from(v: $t) -> Self {
                Self::$constructor(v)
            }
        }
    };
}

impl_from_t!(bool, from_bool);
impl_from_t!(i8, from_i8);
impl_from_t!(i16, from_i16);
impl_from_t!(i32, from_i32);
impl_from_t!(i64, from_i64);
impl_from_t!(i128, from_i128);
impl_from_t!(u8, from_u8);
impl_from_t!(u16, from_u16);
impl_from_t!(u32, from_u32);
impl_from_t!(u64, from_u64);
impl_from_t!(u128, from_u128);
impl_from_t!(f32, from_f32);
impl_from_t!(f64, from_f64);
impl_from_t!(Decimal, from_decimal);
impl_from_t!(char, from_char);
impl_from_t!(String, from_string);
impl_from_t!(NaiveDate, from_date);
impl_from_t!(NaiveTime, from_time);
impl_from_t!(NaiveDateTime, from_timestamp);
impl_from_t!(DateTime<Utc>, from_timestampz);
impl_from_t!(Vec<u8>, from_bytes);
impl_from_t!(Uuid, from_uuid);
impl_from_t!(JsonValue, from_json);

impl From<&str> for DbValue {
    #[inline]
    fn from(val: &str) -> Self {
        Self::from_string(val.to_string())
    }
}

// Implementing From<Option<T>>
macro_rules! impl_from_option_t {
    ($t:ty, $constructor:ident) => {
        impl From<Option<$t>> for DbValue {
            #[inline]
            fn from(v: Option<$t>) -> Self {
                match v {
                    Some(v) => Self::$constructor(v),
                    None => Self::from_null(),
                }
            }
        }
    };
}

impl_from_option_t!(bool, from_bool);
impl_from_option_t!(i8, from_i8);
impl_from_option_t!(i16, from_i16);
impl_from_option_t!(i32, from_i32);
impl_from_option_t!(i64, from_i64);
impl_from_option_t!(i128, from_i128);
impl_from_option_t!(u8, from_u8);
impl_from_option_t!(u16, from_u16);
impl_from_option_t!(u32, from_u32);
impl_from_option_t!(u64, from_u64);
impl_from_option_t!(u128, from_u128);
impl_from_option_t!(f32, from_f32);
impl_from_option_t!(f64, from_f64);
impl_from_option_t!(Decimal, from_decimal);
impl_from_option_t!(char, from_char);
impl_from_option_t!(String, from_string);
impl_from_option_t!(NaiveDate, from_date);
impl_from_option_t!(NaiveTime, from_time);
impl_from_option_t!(NaiveDateTime, from_timestamp);
impl_from_option_t!(DateTime<Utc>, from_timestampz);
impl_from_option_t!(Vec<u8>, from_bytes);
impl_from_option_t!(Uuid, from_uuid);
impl_from_option_t!(JsonValue, from_json);

impl From<Option<&str>> for DbValue {
    #[inline]
    fn from(val: Option<&str>) -> Self {
        match val {
            Some(v) => Self::from_string(v.to_string()),
            None => Self::from_null(),
        }
    }
}

macro_rules! impl_try_from {
    ($t:ty, $as_fn:ident, $type_name:expr, copy) => {
        impl TryFrom<&DbValue> for $t {
            type Error = DbValueConversionError;
            #[inline]
            fn try_from(value: &DbValue) -> Result<Self, Self::Error> {
                value.$as_fn().ok_or(DbValueConversionError { expected: $type_name })
            }
        }

        impl TryFrom<DbValue> for $t {
            type Error = DbValueConversionError;
            #[inline]
            fn try_from(value: DbValue) -> Result<Self, Self::Error> {
                <$t as TryFrom<&DbValue>>::try_from(&value)
            }
        }
    };
    ($t:ty, $as_fn:ident, $type_name:expr, clone) => {
        impl TryFrom<&DbValue> for $t {
            type Error = DbValueConversionError;
            #[inline]
            fn try_from(value: &DbValue) -> Result<Self, Self::Error> {
                value
                    .$as_fn()
                    .map(|v| v.clone())
                    .ok_or(DbValueConversionError { expected: $type_name })
            }
        }

        impl TryFrom<DbValue> for $t {
            type Error = DbValueConversionError;
            #[inline]
            fn try_from(value: DbValue) -> Result<Self, Self::Error> {
                <$t as TryFrom<&DbValue>>::try_from(&value)
            }
        }
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DbValueConversionError {
    pub expected: &'static str,
}

impl_try_from!(bool, as_bool, "bool", copy);
impl_try_from!(i8, as_i8, "i8", copy);
impl_try_from!(i16, as_i16, "i16", copy);
impl_try_from!(i32, as_i32, "i32", copy);
impl_try_from!(i64, as_i64, "i64", copy);
impl_try_from!(u8, as_u8, "u8", copy);
impl_try_from!(u16, as_u16, "u16", copy);
impl_try_from!(u32, as_u32, "u32", copy);
impl_try_from!(u64, as_u64, "u64", copy);
impl_try_from!(f32, as_f32, "f32", copy);
impl_try_from!(f64, as_f64, "f64", copy);
impl_try_from!(char, as_char, "char", copy);

// Reference-returning decoders: clone to produce owned values.
impl_try_from!(i128, as_i128, "i128", clone);
impl_try_from!(u128, as_u128, "u128", clone);
impl_try_from!(Decimal, as_decimal, "Decimal", clone);
impl_try_from!(String, as_string, "String", clone);
impl_try_from!(NaiveDate, as_date, "NaiveDate", clone);
impl_try_from!(NaiveTime, as_time, "NaiveTime", clone);
impl_try_from!(NaiveDateTime, as_timestamp, "NaiveDateTime", clone);
impl_try_from!(DateTime<Utc>, as_timestampz, "DateTime<Utc>", clone);
impl_try_from!(Uuid, as_uuid, "Uuid", clone);
impl_try_from!(JsonValue, as_json, "JsonValue", clone);

impl TryFrom<&DbValue> for Vec<u8> {
    type Error = DbValueConversionError;
    #[inline]
    fn try_from(value: &DbValue) -> Result<Self, Self::Error> {
        value
            .as_bytes()
            .map(|b| b.to_vec())
            .ok_or(DbValueConversionError { expected: "Vec<u8>" })
    }
}

impl TryFrom<DbValue> for Vec<u8> {
    type Error = DbValueConversionError;
    #[inline]
    fn try_from(value: DbValue) -> Result<Self, Self::Error> {
        Vec::<u8>::try_from(&value)
    }
}