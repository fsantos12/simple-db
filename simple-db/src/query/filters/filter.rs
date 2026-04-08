//! Filter definitions and evaluation nodes for query conditions.
//!
//! This module defines the `Filter` enum which represents the AST for query conditions.
//! Filters can be combined using logical operators (And, Or, Not) to build complex query predicates.

use crate::types::DbValue;
use smol_str::SmolStr;
use smallvec::SmallVec;

/// Type alias for a list of filter predicates (implicit AND logic).
/// Stack-allocated for up to 8 filters; larger queries spill to heap automatically.
pub type FilterDefinition = SmallVec<[Filter; 8]>;

/// Filter AST node representing a single condition or logical operation.
///
/// Filters are combined into a `FilterDefinition` vector where they are evaluated
/// with implicit AND logic. Use `And`, `Or`, and `Not` for explicit logical grouping.
#[derive(Debug, Clone)]
pub enum Filter {
    // --- Null Checks ---
    IsNull(SmolStr),
    IsNotNull(SmolStr),

    // --- Basic Comparisons ---
    Eq(SmolStr, DbValue),
    Neq(SmolStr, DbValue),
    Lt(SmolStr, DbValue),
    Lte(SmolStr, DbValue),
    Gt(SmolStr, DbValue),
    Gte(SmolStr, DbValue),

    // --- Pattern Matching ---
    StartsWith(SmolStr, DbValue),
    NotStartsWith(SmolStr, DbValue),
    EndsWith(SmolStr, DbValue),
    NotEndsWith(SmolStr, DbValue),
    Contains(SmolStr, DbValue),
    NotContains(SmolStr, DbValue),

    // --- Regex Matching ---
    Regex(SmolStr, SmolStr),

    // --- Range Checks ---
    Between(SmolStr, Box<(DbValue, DbValue)>),
    NotBetween(SmolStr, Box<(DbValue, DbValue)>),

    // --- Set Membership ---
    In(SmolStr, Box<Vec<DbValue>>),
    NotIn(SmolStr, Box<Vec<DbValue>>),

    // --- Logical Operators ---
    And(Box<FilterDefinition>),
    Or(Box<FilterDefinition>),
    Not(Box<Filter>),
}

#[cfg(test)]
mod tests { 
    use smallvec::smallvec;
    use super::*;

    #[test]
    fn test_filter_creation_null_check() {
        let filter = Filter::IsNull(SmolStr::new("email"));
        assert!(matches!(filter, Filter::IsNull(_)));

        let filter = Filter::IsNotNull(SmolStr::new("age"));
        assert!(matches!(filter, Filter::IsNotNull(_)));
    }

    #[test]
    fn test_filter_creation_comparisons() {
        let filter = Filter::Eq(SmolStr::new("status"), DbValue::String("active".to_string()));
        assert!(matches!(filter, Filter::Eq(_, _)));

        let filter = Filter::Lt(SmolStr::new("age"), DbValue::I32(18));
        assert!(matches!(filter, Filter::Lt(_, _)));

        let filter = Filter::Gte(SmolStr::new("score"), DbValue::I32(100));
        assert!(matches!(filter, Filter::Gte(_, _)));
    }

    #[test]
    fn test_filter_creation_pattern_matching() {
        let filter = Filter::StartsWith(SmolStr::new("name"), DbValue::String("A".to_string()));
        assert!(matches!(filter, Filter::StartsWith(_, _)));

        let filter = Filter::Contains(SmolStr::new("text"), DbValue::String("keyword".to_string()));
        assert!(matches!(filter, Filter::Contains(_, _)));

        let filter = Filter::EndsWith(SmolStr::new("email"), DbValue::String("@example.com".to_string()));
        assert!(matches!(filter, Filter::EndsWith(_, _)));
    }

    #[test]
    fn test_filter_creation_range() {
        let filter = Filter::Between(
            SmolStr::new("age"),
            Box::new((DbValue::I32(18), DbValue::I32(65)))
        );
        assert!(matches!(filter, Filter::Between(_, _)));

        let filter = Filter::NotBetween(
            SmolStr::new("score"),
            Box::new((DbValue::I32(0), DbValue::I32(50)))
        );
        assert!(matches!(filter, Filter::NotBetween(_, _)));
    }

    #[test]
    fn test_filter_creation_set_membership() {
        let values = vec![
            DbValue::String("active".to_string()),
            DbValue::String("pending".to_string()),
            DbValue::String("archived".to_string()),
        ];
        let filter = Filter::In(SmolStr::new("status"), Box::new(values));
        assert!(matches!(filter, Filter::In(_, _)));

        let values = vec![DbValue::I32(1), DbValue::I32(2)];
        let filter = Filter::NotIn(SmolStr::new("role_id"), Box::new(values));
        assert!(matches!(filter, Filter::NotIn(_, _)));
    }

    #[test]
    fn test_filter_creation_logical_operators() {
        let filters = vec![
            Filter::Eq(SmolStr::new("status"), DbValue::String("active".to_string())),
            Filter::Gte(SmolStr::new("age"), DbValue::I32(18)),
        ];
        let and_filter = Filter::And(Box::new(filters.clone().into()));
        assert!(matches!(and_filter, Filter::And(_)));

        let or_filter = Filter::Or(Box::new(filters.into()));
        assert!(matches!(or_filter, Filter::Or(_)));
    }

    #[test]
    fn test_filter_creation_not() {
        let inner = Filter::Eq(SmolStr::new("active"), DbValue::Bool(true));
        let not_filter = Filter::Not(Box::new(inner));
        assert!(matches!(not_filter, Filter::Not(_)));
    }

    #[test]
    fn test_filter_creation_regex() {
        let filter = Filter::Regex(SmolStr::new("email"), SmolStr::new(r".*@example\\.com$"));
        assert!(matches!(filter, Filter::Regex(_, _)));
    }

    #[test]
    fn test_complex_nested_filters() {
        // (status = 'active' AND age >= 18) OR role = 'admin'
        let active_and_adult = Filter::And(Box::new(smallvec![
            Filter::Eq(SmolStr::new("status"), DbValue::String("active".to_string())),
            Filter::Gte(SmolStr::new("age"), DbValue::I32(18)),
        ]));

        let is_admin = Filter::Eq(SmolStr::new("role"), DbValue::String("admin".to_string()));

        let complex = Filter::Or(Box::new(smallvec![active_and_adult, is_admin]));
        assert!(matches!(complex, Filter::Or(_)));
    }
}

