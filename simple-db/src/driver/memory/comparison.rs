//! Type-safe comparison and equality functions for database values.
//!
//! Provides strict comparison and equality checking for `DbValue` types,
//! ensuring only compatible types are compared and returning `None` for
//! incompatible type combinations.

use std::cmp::Ordering;

use crate::types::DbValue;

pub fn strict_partial_cmp(left: &DbValue, right: &DbValue) -> Option<Ordering> {
    macro_rules! compare_inner {
        ($($variant:ident),+ $(,)?) => {
            match (left, right) {
                $(
                    // 'a' and 'b' are &Option<T>. Rust knows how to compare Options!
                    (DbValue::$variant(a), DbValue::$variant(b)) => a.partial_cmp(b),
                )+
                // If they are different variants, or uncomparable types (like Json), return None
                _ => None,
            }
        };
    }

    compare_inner!(
        I8, I16, I32, I64, I128,
        U8, U16, U32, U64, U128,
        F32, F64, Decimal,
        Bool, Char, String,
        Date, Time, Timestamp, Timestamptz
    )
}

pub fn strict_eq(left: &DbValue, right: &DbValue) -> bool {
    macro_rules! eq_inner {
        ($($variant:ident),+ $(,)?) => {
            match (left, right) {
                $(
                    (DbValue::$variant(a), DbValue::$variant(b)) => a == b,
                )+
                // Types like Uuid, Bytes, and Json can go here if you want strict equality for them
                (DbValue::Uuid(a), DbValue::Uuid(b)) => a == b,
                (DbValue::Bytes(a), DbValue::Bytes(b)) => a == b,
                (DbValue::Json(a), DbValue::Json(b)) => a == b,
                _ => false, // Different variants are not strictly equal
            }
        };
    }

    eq_inner!(
        I8, I16, I32, I64, I128,
        U8, U16, U32, U64, U128,
        F32, F64, Decimal,
        Bool, Char, String,
        Date, Time, Timestamp, Timestamptz
    )
}