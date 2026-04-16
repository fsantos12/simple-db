//! **Compact Database Value Representation**
//!
//! This module provides `DbValue`, a memory-efficient tagged union for database values.
//! It combines type information and value data into a single 64-bit word using bit-packing:
//!
//! - **High 16 bits (Tag):** Type identifier + storage category (inline/boxed)
//! - **Low 48 bits (Payload):** Either the value bits (inline) or a pointer (boxed)
//!
//! ## Memory Layout
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │ Tag (16 bits)      │ Payload (48 bits)                  │
//! ├─────────────────────────────────────────────────────────┤
//! │ Category | Type    │ Value bits or pointer address      │
//! └─────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Categories
//!
//! - **INLINE:** Value data fits in 48 bits (8 bytes, bools, floats, chars, etc.)
//! - **BOXED:** Value requires heap allocation (strings, decimals, dates, JSON, etc.)
//!
//! ## Performance Benefits
//!
//! - Small values (i8-u32, bool, char) have zero allocation overhead
//! - No vtable or enum discriminant overhead
//! - 8-byte footprint enables cache-efficient storage of millions of values
//! - Compatible with `From<T>` and `TryFrom<&DbValue>` for ergonomic conversions
//!
//! ## Architecture Requirement
//!
//! Assumes x86-64 with canonical 48-bit user-space pointers. Will not work on 32-bit systems.

use std::ptr::NonNull;

use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use serde_json::Value as JsonValue;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::error::DbError;

/// Bit shift to extract tag from the 64-bit encoding. Tag resides in bits 48-63.
const TAG_SHIFT:    u64 = 48;

/// Mask for payload bits (0-47). Value data or pointer address fits here.
const PAYLOAD_MASK: u64 = (1 << TAG_SHIFT) - 1;

/// Mask to extract category from tag (bits 14-15). Determines inline vs. boxed.
const CATEGORY_MASK: u64 = 0b1100000000000000;

/// Mask to extract type identifier from tag (bits 0-13). 14 bits for 24+ types.
const TYPE_MASK:     u64 = 0b0011111111111111;

/// Category: inline value (no heap allocation required).
const CATEGORY_INLINE: u64 = 0b0000000000000000;

/// Category: boxed value (stores heap-allocated pointer in payload).
const CATEGORY_BOXED:  u64 = 0b0100000000000000;

/// Type: NULL / None value.
const TYPE_NULL:       u64 = 0;
/// Type: Boolean (1 bit payload: 0 = false, 1 = true).
const TYPE_BOOL:       u64 = 1;
/// Type: Signed 8-bit integer (inline, fits in 48 bits).
const TYPE_I8:         u64 = 2;
/// Type: Signed 16-bit integer (inline, fits in 48 bits).
const TYPE_I16:        u64 = 3;
/// Type: Signed 32-bit integer (inline, fits in 48 bits).
const TYPE_I32:        u64 = 4;
/// Type: Signed 64-bit integer (inline if -2^47..2^47-1, else boxed).
const TYPE_I64:        u64 = 5;
/// Type: Signed 128-bit integer (boxed).
const TYPE_I128:       u64 = 6;
/// Type: Unsigned 8-bit integer (inline, fits in 48 bits).
const TYPE_U8:         u64 = 7;
/// Type: Unsigned 16-bit integer (inline, fits in 48 bits).
const TYPE_U16:        u64 = 8;
/// Type: Unsigned 32-bit integer (inline, fits in 48 bits).
const TYPE_U32:        u64 = 9;
/// Type: Unsigned 64-bit integer (inline if < 2^48, else boxed).
const TYPE_U64:        u64 = 10;
/// Type: Unsigned 128-bit integer (boxed).
const TYPE_U128:       u64 = 11;
/// Type: IEEE 32-bit float (inline, bits packed in payload).
const TYPE_F32:        u64 = 12;
/// Type: IEEE 64-bit float (boxed, requires heap allocation).
const TYPE_F64:        u64 = 13;
/// Type: Decimal (arbitrary precision, boxed).
const TYPE_DECIMAL:    u64 = 14;
/// Type: Unicode character (inline, fits 21 bits in 48-bit payload).
const TYPE_CHAR:       u64 = 15;
/// Type: UTF-8 string (heap-allocated).
const TYPE_STRING:     u64 = 16;
/// Type: calendar date without time (boxed).
const TYPE_DATE:       u64 = 17;
/// Type: Time of day without date (boxed).
const TYPE_TIME:       u64 = 18;
/// Type: Date and time (naive, no timezone, boxed).
const TYPE_TIMESTAMP:  u64 = 19;
/// Type: Date and time with UTC timezone (boxed).
const TYPE_TIMESTAMPZ: u64 = 20;
/// Type: Raw byte slice (heap-allocated Vec<u8>).
const TYPE_BYTES:      u64 = 21;
/// Type: UUID / GUID (128-bit unique identifier, boxed).
const TYPE_UUID:       u64 = 22;
/// Type: JSON value (serde_json::Value, boxed).
const TYPE_JSON:       u64 = 23;

/// A 64-bit tagged value for efficient database operations.
///
/// Layout: `(tag << 48) | payload` where tag encodes type + category.
///
/// # Examples
///
/// ```rust
/// use simple_db_core::DbValue;
///
/// // Inline value (no allocation)
/// let i32_val = DbValue::from_i32(42);
/// assert_eq!(i32_val.as_i32(), Some(42));
///
/// // Boxed value (heap allocation)
/// let str_val = DbValue::from_string("hello");
/// assert_eq!(str_val.as_string(), Some("hello"));
///
/// // Use .into() for convenience
/// let val: DbValue = "world".into();
/// assert_eq!(val.as_string(), Some("world"));
/// ```
#[derive(Debug)]
pub struct DbValue(u64);

impl DbValue {
    /// Combines category and type into a 16-bit tag.
    ///
    /// Asserts that both values fit in their respective masks.
    #[inline]
    fn mk_tag(category: u64, ty: u64) -> u64 {
        debug_assert_eq!(category & !CATEGORY_MASK, 0);
        debug_assert_eq!(ty & !TYPE_MASK, 0);
        category | ty
    }

    /// Constructs a `DbValue` from a 16-bit tag and 48-bit payload.
    ///
    /// Combines them into a single 64-bit word: `(tag << 48) | payload`.
    #[inline]
    fn from_tag_and_payload(tag: u64, payload: u64) -> Self {
        debug_assert!(tag < (1 << 16));
        debug_assert_eq!(payload & !PAYLOAD_MASK, 0);
        Self((tag << TAG_SHIFT) | payload)
    }

    /// Stores a signed 48-bit integer in the payload using 2's complement.
    ///
    /// Used for i64s that fit within the 48-bit range.
    #[inline]
    fn from_tag_and_i48(tag: u64, val: i64) -> Self {
        // Store in 2's complement within 48 bits.
        let payload = (val as u64) & PAYLOAD_MASK;
        Self::from_tag_and_payload(tag, payload)
    }

    /// Stores an unsigned 48-bit integer in the payload.
    ///
    /// Debug assertions ensure the value actually fits in 48 bits.
    #[inline]
    fn from_tag_and_u48(tag: u64, val: u64) -> Self {
        debug_assert_eq!(val & !PAYLOAD_MASK, 0, "value does not fit in 48 bits");
        Self::from_tag_and_payload(tag, val & PAYLOAD_MASK)
    }

    /// Boxes a value and stores its pointer in the payload.
    ///
    /// # Architecture Note
    /// Assumes x86-64 canonical 48-bit user-space pointers. If the pointer
    /// exceeds 48 bits, this will panic in debug builds and silently truncate
    /// (causing UB) in release builds.
    ///
    /// # Panics
    /// If `Box::into_raw()` returns null (should never happen).
    #[inline]
    fn from_tag_and_boxed<T>(tag: u64, val: T) -> Self {
        let raw = Box::into_raw(Box::new(val));
        let ptr = NonNull::new(raw).expect("Box::into_raw returned null");
        let addr = ptr.as_ptr() as usize as u64;
        debug_assert_eq!(addr & !PAYLOAD_MASK, 0, "pointer does not fit in 48 bits");
        Self::from_tag_and_payload(tag, addr & PAYLOAD_MASK)
    }

    /// Extracts the 16-bit tag from this value (bits 48-63).
    #[inline]
    fn tag(&self) -> u64 {
        self.0 >> TAG_SHIFT
    }

    /// Extracts the category from the tag (INLINE or BOXED).
    #[inline]
    fn category(&self) -> u64 {
        self.tag() & CATEGORY_MASK
    }

    /// Extracts the type identifier from the tag (distinguishes int/string/etc.).
    #[inline]
    fn ty(&self) -> u64 {
        self.tag() & TYPE_MASK
    }

    /// Extracts the 48-bit payload (value bits or pointer address).
    #[inline]
    fn payload(&self) -> u64 {
        self.0 & PAYLOAD_MASK
    }

    /// Recovers a signed 48-bit integer from the payload with sign extension.
    ///
    /// Uses sign-extension to recover negative numbers stored in 2's complement.
    #[inline]
    fn payload_as_i64_i48(&self) -> i64 {
        // Sign-extend from 48-bit 2's complement.
        let p = self.payload();
        let sign_bit = 1u64 << 47;
        if (p & sign_bit) != 0 {
            // Negative: fill upper 16 bits with 1s
            (p | !PAYLOAD_MASK) as i64
        } else {
            // Positive: upper bits remain 0
            p as i64
        }
    }

    /// Dereferences the payload as a pointer to `T`.
    ///
    /// # Safety
    /// The payload **must** be a valid pointer to an allocated `T`.
    /// Called by `Drop` and `Clone` impls only, where the type is known.
    #[inline]
    unsafe fn payload_as_ref<T>(&self) -> &T {
        unsafe { &*(self.payload() as usize as *const T) }
    }

    // =========================================================================
    // INLINE VALUE CONSTRUCTORS (zero allocation, value bits stored directly)
    // =========================================================================

    /// Creates a NULL value.
    #[inline]
    pub fn from_null() -> Self {
        Self::from_tag_and_payload(Self::mk_tag(CATEGORY_INLINE, TYPE_NULL), 0)
    }

    /// Checks if this value is NULL.
    #[inline]
    pub fn is_null(&self) -> bool {
        self.ty() == TYPE_NULL
    }

    /// Creates a boolean value.
    #[inline]
    pub fn from_bool(val: bool) -> Self {
        Self::from_tag_and_u48(Self::mk_tag(CATEGORY_INLINE, TYPE_BOOL), if val { 1 } else { 0 })
    }

    /// Retrieves this value as a boolean, or `None` if the type doesn't match.
    #[inline]
    pub fn as_bool(&self) -> Option<bool> {
        (self.ty() == TYPE_BOOL).then(|| self.payload() != 0)
    }

    #[inline]
    pub fn is_bool(&self) -> bool {
        self.ty() == TYPE_BOOL
    }

    /// Creates a signed 8-bit integer.
    #[inline]
    pub fn from_i8(val: i8) -> Self {
        Self::from_tag_and_i48(Self::mk_tag(CATEGORY_INLINE, TYPE_I8), val as i64)
    }

    #[inline]
    pub fn as_i8(&self) -> Option<i8> {
        (self.ty() == TYPE_I8).then(|| self.payload_as_i64_i48() as i8)
    }

    #[inline]
    pub fn is_i8(&self) -> bool {
        self.ty() == TYPE_I8
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
    pub fn is_i16(&self) -> bool {
        self.ty() == TYPE_I16
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
    pub fn is_i32(&self) -> bool {
        self.ty() == TYPE_I32
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
    pub fn is_u8(&self) -> bool {
        self.ty() == TYPE_U8
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
    pub fn is_u16(&self) -> bool {
        self.ty() == TYPE_U16
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
    pub fn is_u32(&self) -> bool {
        self.ty() == TYPE_U32
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
    pub fn is_f32(&self) -> bool {
        self.ty() == TYPE_F32
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

    #[inline]
    pub fn is_char(&self) -> bool {
        self.ty() == TYPE_CHAR
    }

    // =========================================================================
    // BOXED VALUE CONSTRUCTORS (heap-allocated or conditionally boxed)
    // =========================================================================

    /// Creates a signed 64-bit integer.
    ///
    /// If the value fits in 48 bits, it's stored inline. Otherwise, boxed.
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

    /// Retrieves this value as a signed 64-bit integer.
    ///
    /// Returns `None` if the type doesn't match or category is invalid.
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
    pub fn is_i64(&self) -> bool {
        self.ty() == TYPE_I64
    }

    /// Creates an unsigned 64-bit integer.
    ///
    /// If the value fits in 48 bits, it's stored inline. Otherwise, boxed.
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
    pub fn is_u64(&self) -> bool {
        self.ty() == TYPE_U64
    }

    /// Creates an IEEE 64-bit float (always boxed).
    ///
    /// f64 values cannot be stored inline and are always heap-allocated.
    #[inline]
    pub fn from_f64(val: f64) -> Self {
        Self::from_tag_and_boxed(Self::mk_tag(CATEGORY_BOXED, TYPE_F64), val.to_bits())
    }

    /// Retrieves this value as an f64, or `None` if the type doesn't match.
    #[inline]
    pub fn as_f64(&self) -> Option<f64> {
        if self.ty() != TYPE_F64 || self.category() != CATEGORY_BOXED {
            return None;
        }
        let bits = unsafe { *self.payload_as_ref::<u64>() };
        Some(f64::from_bits(bits))
    }

    #[inline]
    pub fn is_f64(&self) -> bool {
        self.ty() == TYPE_F64
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
    pub fn is_i128(&self) -> bool {
        self.ty() == TYPE_I128
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
    pub fn is_u128(&self) -> bool {
        self.ty() == TYPE_U128
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
    pub fn is_decimal(&self) -> bool {
        self.ty() == TYPE_DECIMAL
    }

    /// Creates a UTF-8 string value (always boxed).
    ///
    /// Accepts anything convertible to `String`.
    #[inline]
    pub fn from_string(val: impl Into<String>) -> Self {
        Self::from_tag_and_boxed(Self::mk_tag(CATEGORY_BOXED, TYPE_STRING), val.into())
    }

    /// Retrieves this value as a string slice, or `None` if the type doesn't match.
    #[inline]
    pub fn as_string(&self) -> Option<&str> {
        if self.ty() != TYPE_STRING || self.category() != CATEGORY_BOXED {
            return None;
        }
        Some(unsafe { self.payload_as_ref::<String>().as_str() })
    }

    #[inline]
    pub fn is_string(&self) -> bool {
        self.ty() == TYPE_STRING
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
    pub fn is_bytes(&self) -> bool {
        self.ty() == TYPE_BYTES
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
    pub fn is_uuid(&self) -> bool {
        self.ty() == TYPE_UUID
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
    pub fn is_json(&self) -> bool {
        self.ty() == TYPE_JSON
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
    pub fn is_date(&self) -> bool {
        self.ty() == TYPE_DATE
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
    pub fn is_time(&self) -> bool {
        self.ty() == TYPE_TIME
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
    pub fn is_timestamp(&self) -> bool {
        self.ty() == TYPE_TIMESTAMP
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

    #[inline]
    pub fn is_timestampz(&self) -> bool {
        self.ty() == TYPE_TIMESTAMPZ
    }

    /// Returns a human-readable string of the current value's type
    #[inline]
    pub fn type_name(&self) -> &'static str {
        match self.ty() {
            TYPE_NULL => "Null",
            TYPE_BOOL => "bool",
            TYPE_I8 => "i8",
            TYPE_I16 => "i16",
            TYPE_I32 => "i32",
            TYPE_I64 => "i64",
            TYPE_I128 => "i128",
            TYPE_U8 => "u8",
            TYPE_U16 => "u16",
            TYPE_U32 => "u32",
            TYPE_U64 => "u64",
            TYPE_U128 => "u128",
            TYPE_F32 => "f32",
            TYPE_F64 => "f64",
            TYPE_DECIMAL => "Decimal",
            TYPE_CHAR => "char",
            TYPE_STRING => "String",
            TYPE_DATE => "NaiveDate",
            TYPE_TIME => "NaiveTime",
            TYPE_TIMESTAMP => "NaiveDateTime",
            TYPE_TIMESTAMPZ => "DateTime<Utc>",
            TYPE_BYTES => "Vec<u8>",
            TYPE_UUID => "Uuid",
            TYPE_JSON => "JsonValue",
            _ => "Unknown",
        }
    }
}

/// Memory safety: deallocates boxed values based on their type tag.
impl Drop for DbValue {
    fn drop(&mut self) {
        // Only boxed values need deallocation; inline values have no heap allocation.
        if self.category() != CATEGORY_BOXED { return; }
        // Reconstruct the original Box<T> and let it drop naturally.
        // The type tag tells us which type was allocated.
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

/// Deep copy: inline values are bitwise copied; boxed values are re-allocated.
impl Clone for DbValue {
    fn clone(&self) -> Self {
        // Inline values can be safely copied bitwise.
        if self.category() != CATEGORY_BOXED { return Self(self.0); }
        // Boxed values must be deep-cloned to avoid double-free and dangling pointers.
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

/// Compares two `DbValue`s for equality.
///
/// Types must match. Equality is determined by the actual value, not the bit representation.
impl PartialEq for DbValue {
    fn eq(&self, other: &Self) -> bool {
        // Different types are never equal.
        if self.ty() != other.ty() { return false; }
        // Compare values based on their actual type.
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

impl PartialOrd for DbValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // Different types are never equal.
        if self.ty() != other.ty() { return None; }
        // Compare values based on their actual type.
        match self.ty() {
            TYPE_NULL => Some(std::cmp::Ordering::Equal),
            TYPE_BOOL => self.as_bool().partial_cmp(&other.as_bool()),
            TYPE_I8 => self.as_i8().partial_cmp(&other.as_i8()),
            TYPE_I16 => self.as_i16().partial_cmp(&other.as_i16()),
            TYPE_I32 => self.as_i32().partial_cmp(&other.as_i32()),
            TYPE_I64 => self.as_i64().partial_cmp(&other.as_i64()),
            TYPE_I128 => self.as_i128().partial_cmp(&other.as_i128()),
            TYPE_U8 => self.as_u8().partial_cmp(&other.as_u8()),
            TYPE_U16 => self.as_u16().partial_cmp(&other.as_u16()),
            TYPE_U32 => self.as_u32().partial_cmp(&other.as_u32()),
            TYPE_U64 => self.as_u64().partial_cmp(&other.as_u64()),
            TYPE_U128 => self.as_u128().partial_cmp(&other.as_u128()),
            TYPE_F32 => self.as_f32().partial_cmp(&other.as_f32()),
            TYPE_F64 => self.as_f64().partial_cmp(&other.as_f64()),
            TYPE_DECIMAL => self.as_decimal().partial_cmp(&other.as_decimal()),
            TYPE_CHAR => self.as_char().partial_cmp(&other.as_char()),
            TYPE_STRING => self.as_string().partial_cmp(&other.as_string()),
            TYPE_DATE => self.as_date().partial_cmp(&other.as_date()),
            TYPE_TIME => self.as_time().partial_cmp(&other.as_time()),
            TYPE_TIMESTAMP => self.as_timestamp().partial_cmp(&other.as_timestamp()),
            TYPE_TIMESTAMPZ => self.as_timestampz().partial_cmp(&other.as_timestampz()),
            TYPE_BYTES => self.as_bytes().partial_cmp(&other.as_bytes()),
            TYPE_UUID => self.as_uuid().partial_cmp(&other.as_uuid()),
            TYPE_JSON => None, // JSON values are not ordered
            _ => None,
        }
    }
}

/// Macro to implement `From<T> for DbValue` for all value types.
///
/// Automatically generates: `impl From<i32> for DbValue { fn from(v) {...} }`
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

/// Macro to implement `From<Option<T>> for DbValue` for all value types.
///
/// Automatically generates: `impl From<Option<i32>> for DbValue { ... }`
/// Maps `None` to `DbValue::from_null()`.
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

/// Macro to implement `TryFrom<&DbValue> for T` with type mismatch error handling.
///
/// Two variants:
/// - `copy`: Type is Copy (i32, bool, etc.), can extract by value
/// - `clone`: Type requires cloning (Decimal, String, etc.)
macro_rules! impl_try_from {
    ($t:ty, $as_fn:ident, $type_name:expr, copy) => {
        /// Try to convert a reference to a `DbValue` into the target type.
        impl TryFrom<&DbValue> for $t {
            type Error = DbError;
            #[inline]
            fn try_from(value: &DbValue) -> Result<Self, Self::Error> {
                value.$as_fn().ok_or_else(|| DbError::TypeMismatch {
                    expected: $type_name.to_string(),
                    found: value.type_name().to_string(),
                })
            }
        }

        /// Try to convert an owned `DbValue` into the target type.
        impl TryFrom<DbValue> for $t {
            type Error = DbError;
            #[inline]
            fn try_from(value: DbValue) -> Result<Self, Self::Error> {
                <$t as TryFrom<&DbValue>>::try_from(&value)
            }
        }
    };
    ($t:ty, $as_fn:ident, $type_name:expr, clone) => {
        impl TryFrom<&DbValue> for $t {
            type Error = DbError;
            #[inline]
            fn try_from(value: &DbValue) -> Result<Self, Self::Error> {
                value
                    .$as_fn()
                    .map(|v| v.clone())
                    .ok_or_else(|| DbError::TypeMismatch {
                        expected: $type_name.to_string(),
                        found: value.type_name().to_string(),
                    })
            }
        }

        impl TryFrom<DbValue> for $t {
            type Error = DbError;
            #[inline]
            fn try_from(value: DbValue) -> Result<Self, Self::Error> {
                <$t as TryFrom<&DbValue>>::try_from(&value)
            }
        }
    };
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
impl_try_from!(i128, as_i128, "i128", clone);
impl_try_from!(u128, as_u128, "u128", clone);
impl_try_from!(Decimal, as_decimal, "Decimal", clone);
impl_try_from!(NaiveDate, as_date, "NaiveDate", clone);
impl_try_from!(NaiveTime, as_time, "NaiveTime", clone);
impl_try_from!(NaiveDateTime, as_timestamp, "NaiveDateTime", clone);
impl_try_from!(DateTime<Utc>, as_timestampz, "DateTime<Utc>", clone);
impl_try_from!(Uuid, as_uuid, "Uuid", clone);
impl_try_from!(JsonValue, as_json, "JsonValue", clone);

impl TryFrom<&DbValue> for String {
    type Error = DbError;
    #[inline]
    fn try_from(value: &DbValue) -> Result<Self, Self::Error> {
        value
            .as_string()
            .map(|s| s.to_string())
            .ok_or_else(|| DbError::TypeMismatch {
                expected: "String".to_string(),
                found: value.type_name().to_string(),
            })
    }
}

impl TryFrom<DbValue> for String {
    type Error = DbError;
    #[inline]
    fn try_from(value: DbValue) -> Result<Self, Self::Error> {
        String::try_from(&value)
    }
}

impl TryFrom<&DbValue> for Vec<u8> {
    type Error = DbError;
    #[inline]
    fn try_from(value: &DbValue) -> Result<Self, Self::Error> {
        value
            .as_bytes()
            .map(|b| b.to_vec())
            .ok_or_else(|| DbError::TypeMismatch {
                expected: "Vec<u8>".to_string(),
                found: value.type_name().to_string(),
            })
    }
}

impl TryFrom<DbValue> for Vec<u8> {
    type Error = DbError;
    #[inline]
    fn try_from(value: DbValue) -> Result<Self, Self::Error> {
        Vec::<u8>::try_from(&value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;
    use std::str::FromStr;
    use uuid::Uuid;
    use chrono::{NaiveDate, NaiveTime, TimeZone, Utc};
    use serde_json::json;

    // =========================================================================
    // INLINE VALUE TESTS (no allocation)
    // =========================================================================

    /// Tests NULL value creation and type mismatch.
    #[test]
    fn test_null() {
        let val = DbValue::from_null();
        assert!(val.is_null());
        assert_eq!(val.type_name(), "Null");
        // Reading as another type should return None
        assert_eq!(val.as_bool(), None);
    }

    #[test]
    fn test_bool() {
        let v_true = DbValue::from_bool(true);
        let v_false = DbValue::from_bool(false);
        assert_eq!(v_true.as_bool(), Some(true));
        assert_eq!(v_false.as_bool(), Some(false));
        assert_eq!(v_true.type_name(), "bool");
    }

    #[test]
    fn test_integers_inline() {
        // i8, i16, i32, u8, u16, u32 cabem sempre na payload
        let v_i8 = DbValue::from_i8(-42);
        assert_eq!(v_i8.as_i8(), Some(-42));

        let v_u32 = DbValue::from_u32(4_000_000_000);
        assert_eq!(v_u32.as_u32(), Some(4_000_000_000));
        assert_eq!(v_u32.category(), CATEGORY_INLINE);
    }

    #[test]
    fn test_float_32_and_char() {
        let v_f32 = DbValue::from_f32(3.1415);
        assert_eq!(v_f32.as_f32(), Some(3.1415));

        let v_char = DbValue::from_char('🦀');
        assert_eq!(v_char.as_char(), Some('🦀'));
    }

    // =========================================================================
    // HYBRID VALUE TESTS (types that can be inline OR boxed)
    // =========================================================================

    #[test]
    fn test_i64_boundaries() {
        let v_small = DbValue::from_i64(100_000);
        assert_eq!(v_small.as_i64(), Some(100_000));
        assert_eq!(v_small.category(), CATEGORY_INLINE);

        // Small negative value: testing sign extension
        let v_small_neg = DbValue::from_i64(-100_000);
        assert_eq!(v_small_neg.as_i64(), Some(-100_000));
        assert_eq!(v_small_neg.category(), CATEGORY_INLINE);

        // Very large value (> 48 bits): should be boxed
        let large_val = 1i64 << 50; 
        let v_large = DbValue::from_i64(large_val);
        assert_eq!(v_large.as_i64(), Some(large_val));
        assert_eq!(v_large.category(), CATEGORY_BOXED);

        // Very large negative value
        let large_neg = -(1i64 << 50);
        let v_large_neg = DbValue::from_i64(large_neg);
        assert_eq!(v_large_neg.as_i64(), Some(large_neg));
        assert_eq!(v_large_neg.category(), CATEGORY_BOXED);
    }

    #[test]
    fn test_u64_boundaries() {
        let v_small = DbValue::from_u64(999_999);
        assert_eq!(v_small.as_u64(), Some(999_999));
        assert_eq!(v_small.category(), CATEGORY_INLINE);

        // Very large value: boxed
        let large_val = 1u64 << 55;
        let v_large = DbValue::from_u64(large_val);
        assert_eq!(v_large.as_u64(), Some(large_val));
        assert_eq!(v_large.category(), CATEGORY_BOXED);
    }

    // =========================================================================
    // BOXED VALUE TESTS (types always requiring heap allocation)
    // =========================================================================

    #[test]
    fn test_f64() {
        let v_f64 = DbValue::from_f64(2.718281828);
        assert_eq!(v_f64.as_f64(), Some(2.718281828));
        assert_eq!(v_f64.category(), CATEGORY_BOXED);
    }

    #[test]
    fn test_large_integers() {
        let val_i128 = DbValue::from_i128(-999_999_999_999_999_999_999);
        assert_eq!(val_i128.as_i128(), Some(&-999_999_999_999_999_999_999));

        let val_u128 = DbValue::from_u128(u128::MAX);
        assert_eq!(val_u128.as_u128(), Some(&u128::MAX));
    }

    #[test]
    fn test_complex_types() {
        let dec = Decimal::from_str("123.45").unwrap();
        let v_dec = DbValue::from_decimal(dec);
        assert_eq!(v_dec.as_decimal(), Some(&dec));

        let id = Uuid::now_v7();
        let v_uuid = DbValue::from_uuid(id);
        assert_eq!(v_uuid.as_uuid(), Some(&id));

        let j = json!({ "key": "value", "number": 42 });
        let v_json = DbValue::from_json(j.clone());
        assert_eq!(v_json.as_json(), Some(&j));
    }

    #[test]
    fn test_strings_and_bytes() {
        let texto = String::from("Hello, Rust!");
        let v_str = DbValue::from_string(texto.clone());
        assert_eq!(v_str.as_string(), Some(texto.as_str()));

        let bytes = vec![0xDE, 0xAD, 0xBE, 0xEF];
        let v_bytes = DbValue::from_bytes(bytes.clone());
        assert_eq!(v_bytes.as_bytes(), Some(bytes.as_slice()));
    }

    #[test]
    fn test_chrono_dates_and_times() {
        let date = NaiveDate::from_ymd_opt(2026, 4, 10).unwrap();
        let v_date = DbValue::from_date(date);
        assert_eq!(v_date.as_date(), Some(&date));

        let time = NaiveTime::from_hms_opt(10, 30, 0).unwrap();
        let v_time = DbValue::from_time(time);
        assert_eq!(v_time.as_time(), Some(&time));

        let dt = date.and_time(time);
        let v_dt = DbValue::from_timestamp(dt);
        assert_eq!(v_dt.as_timestamp(), Some(&dt));

        let dtz = Utc.from_utc_datetime(&dt);
        let v_dtz = DbValue::from_timestampz(dtz);
        assert_eq!(v_dtz.as_timestampz(), Some(&dtz));
    }

    // =========================================================================
    // BEHAVIOR TESTS (type mismatches, cloning, equality)
    // =========================================================================

    #[test]
    fn test_type_mismatch() {
        let v_str = DbValue::from_string("I am text");
        assert_eq!(v_str.as_i32(), None);
        assert_eq!(v_str.as_f64(), None);
        assert_eq!(v_str.as_bool(), None);
    }

    /// Verifies deep cloning: cloned boxed values have separate allocations.
    #[test]
    fn test_clone_and_equality() {
        let val1 = DbValue::from_string("Clone Test");
        let val2 = val1.clone();
        
        assert_eq!(val1, val2);
        
        let ptr1 = val1.payload();
        let ptr2 = val2.payload();
        assert_ne!(ptr1, ptr2, "Boxed values must be deep-copied with different pointers");
    }
}