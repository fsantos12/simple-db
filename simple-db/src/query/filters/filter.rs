//! Filter definitions and evaluation nodes for query conditions.
//!
//! This module defines the `Filter` enum which represents the AST for query conditions.
//! Filters can be combined using logical operators (And, Or, Not) to build complex query predicates.

use crate::types::DbValue;

pub type FilterDefinition = Vec<Filter>;

/// Filter AST node representing a single condition or logical operation.
///
/// Filters are combined into a `FilterDefinition` vector where they are evaluated
/// with implicit AND logic. Use `And`, `Or`, and `Not` for explicit logical grouping.
#[derive(Debug, Clone)]
pub enum Filter {
    // --- Null Checks ---
    IsNull(Box<String>),
    IsNotNull(Box<String>),

    // --- Basic Comparisons ---
    Eq(Box<String>, DbValue),
    Neq(Box<String>, DbValue),
    Lt(Box<String>, DbValue),
    Lte(Box<String>, DbValue),
    Gt(Box<String>, DbValue),
    Gte(Box<String>, DbValue),

    // --- Pattern Matching ---
    StartsWith(Box<String>, DbValue),
    NotStartsWith(Box<String>, DbValue),
    EndsWith(Box<String>, DbValue),
    NotEndsWith(Box<String>, DbValue),
    Contains(Box<String>, DbValue),
    NotContains(Box<String>, DbValue),

    // --- Regex Matching ---
    Regex(Box<String>, Box<String>),

    // --- Range Checks ---
    Between(Box<String>, Box<(DbValue, DbValue)>),
    NotBetween(Box<String>, Box<(DbValue, DbValue)>),

    // --- Set Membership ---
    In(Box<String>, Box<Vec<DbValue>>),
    NotIn(Box<String>, Box<Vec<DbValue>>),

    // --- Logical Operators ---
    And(Box<FilterDefinition>),
    Or(Box<FilterDefinition>),
    Not(Box<Filter>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_creation_null_check() {
        let filter = Filter::IsNull(Box::new("email".to_string()));
        assert!(matches!(filter, Filter::IsNull(_)));

        let filter = Filter::IsNotNull(Box::new("age".to_string()));
        assert!(matches!(filter, Filter::IsNotNull(_)));
    }

    #[test]
    fn test_filter_creation_comparisons() {
        let filter = Filter::Eq(Box::new("status".to_string()), DbValue::String(Some(Box::new("active".to_string()))));
        assert!(matches!(filter, Filter::Eq(_, _)));

        let filter = Filter::Lt(Box::new("age".to_string()), DbValue::I32(Some(18)));
        assert!(matches!(filter, Filter::Lt(_, _)));

        let filter = Filter::Gte(Box::new("score".to_string()), DbValue::I32(Some(100)));
        assert!(matches!(filter, Filter::Gte(_, _)));
    }

    #[test]
    fn test_filter_creation_pattern_matching() {
        let filter = Filter::StartsWith(Box::new("name".to_string()), DbValue::String(Some(Box::new("A".to_string()))));
        assert!(matches!(filter, Filter::StartsWith(_, _)));

        let filter = Filter::Contains(Box::new("text".to_string()), DbValue::String(Some(Box::new("keyword".to_string()))));
        assert!(matches!(filter, Filter::Contains(_, _)));

        let filter = Filter::EndsWith(Box::new("email".to_string()), DbValue::String(Some(Box::new("@example.com".to_string()))));
        assert!(matches!(filter, Filter::EndsWith(_, _)));
    }

    #[test]
    fn test_filter_creation_range() {
        let filter = Filter::Between(
            Box::new("age".to_string()),
            Box::new((DbValue::I32(Some(18)), DbValue::I32(Some(65))))
        );
        assert!(matches!(filter, Filter::Between(_, _)));

        let filter = Filter::NotBetween(
            Box::new("score".to_string()),
            Box::new((DbValue::I32(Some(0)), DbValue::I32(Some(50))))
        );
        assert!(matches!(filter, Filter::NotBetween(_, _)));
    }

    #[test]
    fn test_filter_creation_set_membership() {
        let values = vec![
            DbValue::String(Some(Box::new("active".to_string()))),
            DbValue::String(Some(Box::new("pending".to_string()))),
            DbValue::String(Some(Box::new("archived".to_string()))),
        ];
        let filter = Filter::In(Box::new("status".to_string()), Box::new(values));
        assert!(matches!(filter, Filter::In(_, _)));

        let values = vec![DbValue::I32(Some(1)), DbValue::I32(Some(2))];
        let filter = Filter::NotIn(Box::new("role_id".to_string()), Box::new(values));
        assert!(matches!(filter, Filter::NotIn(_, _)));
    }

    #[test]
    fn test_filter_creation_logical_operators() {
        let filters = vec![
            Filter::Eq(Box::new("status".to_string()), DbValue::String(Some(Box::new("active".to_string())))),
            Filter::Gte(Box::new("age".to_string()), DbValue::I32(Some(18))),
        ];
        let and_filter = Filter::And(Box::new(filters.clone()));
        assert!(matches!(and_filter, Filter::And(_)));

        let or_filter = Filter::Or(Box::new(filters));
        assert!(matches!(or_filter, Filter::Or(_)));
    }

    #[test]
    fn test_filter_creation_not() {
        let inner = Filter::Eq(Box::new("active".to_string()), DbValue::Bool(Some(true)));
        let not_filter = Filter::Not(Box::new(inner));
        assert!(matches!(not_filter, Filter::Not(_)));
    }

    #[test]
    fn test_filter_creation_regex() {
        let filter = Filter::Regex(Box::new("email".to_string()), Box::new(r".*@example\\.com$".to_string()));
        assert!(matches!(filter, Filter::Regex(_, _)));
    }

    #[test]
    fn test_complex_nested_filters() {
        // (status = 'active' AND age >= 18) OR role = 'admin'
        let active_and_adult = Filter::And(Box::new(vec![
            Filter::Eq(Box::new("status".to_string()), DbValue::String(Some(Box::new("active".to_string())))),
            Filter::Gte(Box::new("age".to_string()), DbValue::I32(Some(18))),
        ]));

        let is_admin = Filter::Eq(Box::new("role".to_string()), DbValue::String(Some(Box::new("admin".to_string()))));

        let complex = Filter::Or(Box::new(vec![active_and_adult, is_admin]));
        assert!(matches!(complex, Filter::Or(_)));
    }
}