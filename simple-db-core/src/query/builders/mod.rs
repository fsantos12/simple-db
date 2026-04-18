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

    // =========================================================================
    // FILTER BUILDER TESTS
    // =========================================================================

    #[test]
    fn test_equality_filter() {
        let filter = FilterBuilder::new()
            .eq("age", 25)
            .build();

        assert_eq!(filter.len(), 1);
        assert!(matches!(filter[0], Filter::Eq(_, _)));
    }

    #[test]
    fn test_inequality_filter() {
        let filter = FilterBuilder::new()
            .neq("status", "inactive")
            .build();

        assert_eq!(filter.len(), 1);
        assert!(matches!(filter[0], Filter::Neq(_, _)));
    }

    #[test]
    fn test_comparison_filters() {
        let filters = FilterBuilder::new()
            .lt("age", 18)
            .lte("price", 100)
            .gt("score", 50)
            .gte("rating", 4)
            .build();

        assert_eq!(filters.len(), 4);
    }

    #[test]
    fn test_null_filters() {
        let filters = FilterBuilder::new()
            .is_null("deleted_at")
            .is_not_null("verified_at")
            .build();

        assert_eq!(filters.len(), 2);
    }

    #[test]
    fn test_string_pattern_filters() {
        let filters = FilterBuilder::new()
            .starts_with("email", "@gmail")
            .ends_with("filename", ".pdf")
            .contains("description", "important")
            .build();

        assert_eq!(filters.len(), 3);
    }

    #[test]
    fn test_range_filter() {
        let filter = FilterBuilder::new()
            .between("age", 18, 65)
            .build();

        assert_eq!(filter.len(), 1);
        assert!(matches!(filter[0], Filter::Between(_, _)));
    }

    #[test]
    fn test_in_filter() {
        let statuses = vec!["active", "pending"];
        let filter = FilterBuilder::new()
            .is_in("status", statuses)
            .build();

        assert_eq!(filter.len(), 1);
    }

    #[test]
    fn test_complex_filters_with_and() {
        let filters = FilterBuilder::new()
            .eq("type", "user")
            .gte("age", 18)
            .is_not_null("email")
            .build();

        assert_eq!(filters.len(), 3);
    }

    #[test]
    fn test_negated_filter() {
        let filter = FilterBuilder::new()
            .not(|b| b.eq("status", "deleted"))
            .build();

        assert!(!filter.is_empty());
    }

    // =========================================================================
    // PROJECTION BUILDER TESTS
    // =========================================================================

    #[test]
    fn test_single_field_projection() {
        let projections = ProjectionBuilder::new()
            .field("name")
            .build();

        assert_eq!(projections.len(), 1);
    }

    #[test]
    fn test_multiple_field_projection() {
        let projections = ProjectionBuilder::new()
            .field("id")
            .field("name")
            .field("email")
            .build();

        assert_eq!(projections.len(), 3);
    }

    #[test]
    fn test_aggregation_count_all() {
        let projections = ProjectionBuilder::new()
            .count_all()
            .build();

        assert_eq!(projections.len(), 1);
    }

    #[test]
    fn test_aggregation_count() {
        let projections = ProjectionBuilder::new()
            .count("id")
            .build();

        assert_eq!(projections.len(), 1);
    }

    #[test]
    fn test_aggregation_sum() {
        let projections = ProjectionBuilder::new()
            .sum("total_price")
            .build();

        assert_eq!(projections.len(), 1);
    }

    #[test]
    fn test_aggregation_avg() {
        let projections = ProjectionBuilder::new()
            .avg("rating")
            .build();

        assert_eq!(projections.len(), 1);
    }

    #[test]
    fn test_aggregation_min_max() {
        let projections = ProjectionBuilder::new()
            .min("price")
            .max("price")
            .build();

        assert_eq!(projections.len(), 2);
    }

    #[test]
    fn test_multiple_aggregates() {
        let projections = ProjectionBuilder::new()
            .sum("total")
            .count_all()
            .build();

        assert_eq!(projections.len(), 2);
    }

    #[test]
    fn test_mixed_fields_and_aggregates() {
        let projections = ProjectionBuilder::new()
            .field("category")
            .count_all()
            .sum("amount")
            .build();

        assert_eq!(projections.len(), 3);
    }

    // =========================================================================
    // SORT BUILDER TESTS
    // =========================================================================

    #[test]
    fn test_ascending_sort() {
        let sorts = SortBuilder::new()
            .asc("name")
            .build();

        assert_eq!(sorts.len(), 1);
    }

    #[test]
    fn test_descending_sort() {
        let sorts = SortBuilder::new()
            .desc("created_at")
            .build();

        assert_eq!(sorts.len(), 1);
    }

    #[test]
    fn test_multiple_sorts() {
        let sorts = SortBuilder::new()
            .asc("category")
            .desc("price")
            .asc("name")
            .build();

        assert_eq!(sorts.len(), 3);
    }

    #[test]
    fn test_null_ordering_asc() {
        let sorts = SortBuilder::new()
            .asc_nulls_first("priority")
            .build();

        assert_eq!(sorts.len(), 1);
    }

    #[test]
    fn test_null_ordering_desc() {
        let sorts = SortBuilder::new()
            .desc_nulls_last("optional_field")
            .build();

        assert_eq!(sorts.len(), 1);
    }

    #[test]
    fn test_random_sort() {
        let sorts = SortBuilder::new()
            .random()
            .build();

        assert_eq!(sorts.len(), 1);
    }

    // =========================================================================
    // GROUP BUILDER TESTS
    // =========================================================================

    #[test]
    fn test_single_group() {
        let groups = GroupBuilder::new()
            .field("category")
            .build();

        assert_eq!(groups.len(), 1);
    }

    #[test]
    fn test_multiple_groups() {
        let groups = GroupBuilder::new()
            .field("category")
            .field("status")
            .field("year")
            .build();

        assert_eq!(groups.len(), 3);
    }

    #[test]
    fn test_group_with_fields_convenience() {
        let groups = GroupBuilder::new()
            .fields(vec!["dept", "role"])
            .build();

        assert_eq!(groups.len(), 2);
    }
}
