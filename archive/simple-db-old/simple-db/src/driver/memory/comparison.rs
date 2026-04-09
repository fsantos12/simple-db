//! Type-safe comparison and equality functions for database values.
//!
//! Provides strict comparison and equality checking for `DbValue` types,
//! ensuring only compatible types are compared and returning `None` for
//! incompatible type combinations.

use std::cmp::Ordering;

use crate::types::{DbValue, DbValueRef};

pub fn strict_partial_cmp(left: &DbValue, right: &DbValue) -> Option<Ordering> {
    match (DbValueRef::from(left), DbValueRef::from(right)) {
        (DbValueRef::Null, DbValueRef::Null) => Some(Ordering::Equal),

        (DbValueRef::I8(a), DbValueRef::I8(b)) => a.partial_cmp(&b),
        (DbValueRef::I16(a), DbValueRef::I16(b)) => a.partial_cmp(&b),
        (DbValueRef::I32(a), DbValueRef::I32(b)) => a.partial_cmp(&b),
        (DbValueRef::I64(a), DbValueRef::I64(b)) => a.partial_cmp(&b),
        (DbValueRef::I128(a), DbValueRef::I128(b)) => a.partial_cmp(&b),

        (DbValueRef::U8(a), DbValueRef::U8(b)) => a.partial_cmp(&b),
        (DbValueRef::U16(a), DbValueRef::U16(b)) => a.partial_cmp(&b),
        (DbValueRef::U32(a), DbValueRef::U32(b)) => a.partial_cmp(&b),
        (DbValueRef::U64(a), DbValueRef::U64(b)) => a.partial_cmp(&b),
        (DbValueRef::U128(a), DbValueRef::U128(b)) => a.partial_cmp(&b),

        (DbValueRef::F32(a), DbValueRef::F32(b)) => a.partial_cmp(&b),
        (DbValueRef::F64(a), DbValueRef::F64(b)) => a.partial_cmp(&b),

        (DbValueRef::Decimal(a), DbValueRef::Decimal(b)) => a.partial_cmp(b),
        (DbValueRef::Bool(a), DbValueRef::Bool(b)) => a.partial_cmp(&b),
        (DbValueRef::Char(a), DbValueRef::Char(b)) => a.partial_cmp(&b),
        (DbValueRef::String(a), DbValueRef::String(b)) => a.partial_cmp(b),

        (DbValueRef::Date(a), DbValueRef::Date(b)) => a.partial_cmp(b),
        (DbValueRef::Time(a), DbValueRef::Time(b)) => a.partial_cmp(b),
        (DbValueRef::Timestamp(a), DbValueRef::Timestamp(b)) => a.partial_cmp(b),
        (DbValueRef::Timestamptz(a), DbValueRef::Timestamptz(b)) => a.partial_cmp(b),

        _ => None,
    }
}

pub fn strict_eq(left: &DbValue, right: &DbValue) -> bool {
    match (DbValueRef::from(left), DbValueRef::from(right)) {
        (DbValueRef::Null, DbValueRef::Null) => true,

        (DbValueRef::I8(a), DbValueRef::I8(b)) => a == b,
        (DbValueRef::I16(a), DbValueRef::I16(b)) => a == b,
        (DbValueRef::I32(a), DbValueRef::I32(b)) => a == b,
        (DbValueRef::I64(a), DbValueRef::I64(b)) => a == b,
        (DbValueRef::I128(a), DbValueRef::I128(b)) => a == b,

        (DbValueRef::U8(a), DbValueRef::U8(b)) => a == b,
        (DbValueRef::U16(a), DbValueRef::U16(b)) => a == b,
        (DbValueRef::U32(a), DbValueRef::U32(b)) => a == b,
        (DbValueRef::U64(a), DbValueRef::U64(b)) => a == b,
        (DbValueRef::U128(a), DbValueRef::U128(b)) => a == b,

        (DbValueRef::F32(a), DbValueRef::F32(b)) => a == b,
        (DbValueRef::F64(a), DbValueRef::F64(b)) => a == b,

        (DbValueRef::Bool(a), DbValueRef::Bool(b)) => a == b,
        (DbValueRef::Char(a), DbValueRef::Char(b)) => a == b,

        (DbValueRef::Decimal(a), DbValueRef::Decimal(b)) => a == b,
        (DbValueRef::String(a), DbValueRef::String(b)) => a == b,
        (DbValueRef::Date(a), DbValueRef::Date(b)) => a == b,
        (DbValueRef::Time(a), DbValueRef::Time(b)) => a == b,
        (DbValueRef::Timestamp(a), DbValueRef::Timestamp(b)) => a == b,
        (DbValueRef::Timestamptz(a), DbValueRef::Timestamptz(b)) => a == b,

        (DbValueRef::Uuid(a), DbValueRef::Uuid(b)) => a == b,
        (DbValueRef::Bytes(a), DbValueRef::Bytes(b)) => a == b,
        (DbValueRef::Json(a), DbValueRef::Json(b)) => a == b,

        _ => false,
    }
}