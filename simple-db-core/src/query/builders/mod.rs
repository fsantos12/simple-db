//! Query clause builders: filters, projections, sorts, and group-by.
//!
//! Each sub-module provides a fluent builder and an associated type alias:
//! - [`filters`] — [`FilterBuilder`] / [`FilterDefinition`] for WHERE predicates
//! - [`projections`] — [`ProjectionBuilder`] / [`ProjectionDefinition`] for SELECT columns
//! - [`sorts`] — [`SortBuilder`] / [`SortDefinition`] for ORDER BY
//! - [`groups`] — [`GroupBuilder`] / [`GroupDefinition`] for GROUP BY

pub mod filters;
pub mod groups;
pub mod projections;
pub mod sorts;

pub use filters::{Filter, FilterBuilder, FilterDefinition};
pub use projections::{Projection, ProjectionBuilder, ProjectionDefinition};
pub use sorts::{Sort, SortBuilder, SortDefinition};
pub use groups::{GroupBuilder, GroupDefinition};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{filter, project, sort, group};

    // =========================================================================
    // FILTER BUILDER TESTS
    // =========================================================================

    #[test]
    fn test_equality_filter() {
        let filter = filter!(eq("age", 25));

        assert_eq!(filter.len(), 1);
        assert!(matches!(filter[0], Filter::Eq(_, _)));
    }

    #[test]
    fn test_inequality_filter() {
        let filter = filter!(neq("status", "inactive"));

        assert_eq!(filter.len(), 1);
        assert!(matches!(filter[0], Filter::Neq(_, _)));
    }

    #[test]
    fn test_comparison_filters() {
        let filters = filter!(lt("age", 18), lte("price", 100), gt("score", 50), gte("rating", 4));

        assert_eq!(filters.len(), 4);
    }

    #[test]
    fn test_null_filters() {
        let filters = filter!(is_null("deleted_at"), is_not_null("verified_at"));

        assert_eq!(filters.len(), 2);
    }

    #[test]
    fn test_string_pattern_filters() {
        let filters = filter!(
            starts_with("email", "@gmail"),
            ends_with("filename", ".pdf"),
            contains("description", "important")
        );

        assert_eq!(filters.len(), 3);
    }

    #[test]
    fn test_range_filter() {
        let filter = filter!(between("age", 18, 65));

        assert_eq!(filter.len(), 1);
        assert!(matches!(filter[0], Filter::Between(_, _)));
    }

    #[test]
    fn test_in_filter() {
        let filter = filter!(is_in("status", vec!["active", "pending"]));

        assert_eq!(filter.len(), 1);
    }

    #[test]
    fn test_complex_filters_with_and() {
        let filters = filter!(eq("type", "user"), gte("age", 18), is_not_null("email"));

        assert_eq!(filters.len(), 3);
    }

    #[test]
    fn test_negated_filter() {
        let filter = filter!(not(filter!(eq("status", "deleted"))));

        assert!(!filter.is_empty());
    }

    // =========================================================================
    // PROJECTION BUILDER TESTS
    // =========================================================================

    #[test]
    fn test_single_field_projection() {
        let projections = project!(field("name"));

        assert_eq!(projections.len(), 1);
    }

    #[test]
    fn test_multiple_field_projection() {
        let projections = project!(field("id"), field("name"), field("email"));

        assert_eq!(projections.len(), 3);
    }

    #[test]
    fn test_aggregation_count_all() {
        let projections = project!(count_all());

        assert_eq!(projections.len(), 1);
    }

    #[test]
    fn test_aggregation_count() {
        let projections = project!(count("id"));

        assert_eq!(projections.len(), 1);
    }

    #[test]
    fn test_aggregation_sum() {
        let projections = project!(sum("total_price"));

        assert_eq!(projections.len(), 1);
    }

    #[test]
    fn test_aggregation_avg() {
        let projections = project!(avg("rating"));

        assert_eq!(projections.len(), 1);
    }

    #[test]
    fn test_aggregation_min_max() {
        let projections = project!(min("price"), max("price"));

        assert_eq!(projections.len(), 2);
    }

    #[test]
    fn test_multiple_aggregates() {
        let projections = project!(sum("total"), count_all());

        assert_eq!(projections.len(), 2);
    }

    #[test]
    fn test_mixed_fields_and_aggregates() {
        let projections = project!(field("category"), count_all(), sum("amount"));

        assert_eq!(projections.len(), 3);
    }

    // =========================================================================
    // SORT BUILDER TESTS
    // =========================================================================

    #[test]
    fn test_ascending_sort() {
        let sorts = sort!(asc("name"));

        assert_eq!(sorts.len(), 1);
    }

    #[test]
    fn test_descending_sort() {
        let sorts = sort!(desc("created_at"));

        assert_eq!(sorts.len(), 1);
    }

    #[test]
    fn test_multiple_sorts() {
        let sorts = sort!(asc("category"), desc("price"), asc("name"));

        assert_eq!(sorts.len(), 3);
    }

    #[test]
    fn test_null_ordering_asc() {
        let sorts = sort!(asc_nulls_first("priority"));

        assert_eq!(sorts.len(), 1);
    }

    #[test]
    fn test_null_ordering_desc() {
        let sorts = sort!(desc_nulls_last("optional_field"));

        assert_eq!(sorts.len(), 1);
    }

    #[test]
    fn test_random_sort() {
        let sorts = sort!(random());

        assert_eq!(sorts.len(), 1);
    }

    // =========================================================================
    // GROUP BUILDER TESTS
    // =========================================================================

    #[test]
    fn test_single_group() {
        let groups = group!("category");

        assert_eq!(groups.len(), 1);
    }

    #[test]
    fn test_multiple_groups() {
        let groups = group!("category", "status", "year");

        assert_eq!(groups.len(), 3);
    }

    #[test]
    fn test_group_with_fields_convenience() {
        let groups = group!("dept", "role");

        assert_eq!(groups.len(), 2);
    }
}
