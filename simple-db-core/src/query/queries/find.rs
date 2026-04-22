use crate::query::{FilterBuilder, FilterDefinition, GroupBuilder, GroupDefinition, ProjectionBuilder, ProjectionDefinition, SortBuilder, SortDefinition};

/// SELECT query builder for reading data from a collection.
///
/// Supports the full SQL SELECT feature set:
/// - Column selection and aggregates (projections)
/// - Row filtering (WHERE clause)
/// - Result ordering (ORDER BY clause)
/// - Aggregate grouping (GROUP BY clause)
/// - Pagination (LIMIT / OFFSET)
///
/// Construct via [`Query::find`](super::Query::find).
///
/// # Example
///
/// ```rust,ignore
/// let query = Query::find("orders")
///     .project(project!(field("customer_id"), sum("total"), count_all()))
///     .filter(filter!(gte("total", 100.0), eq("status", "completed")))
///     .order_by(sort!(desc("total")))
///     .limit(10);
/// ```
#[derive(Debug, Clone)]
pub struct FindQuery {
    pub collection: String,
    pub projections: ProjectionDefinition,
    pub filters: FilterDefinition,
    pub sorts: SortDefinition,
    pub groups: GroupDefinition,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl FindQuery {
    /// Creates a new `FindQuery` targeting the given collection.
    pub fn new<S: Into<String>>(collection: S) -> Self {
        Self {
            collection: collection.into(),
            projections: ProjectionDefinition::new(),
            filters: FilterDefinition::new(),
            sorts: SortDefinition::new(),
            groups: GroupDefinition::new(),
            limit: None,
            offset: None,
        }
    }

    /// Adds column selections and aggregations (SELECT clause) from a pre-built definition.
    ///
    /// Multiple calls accumulate projections — each call appends to the list.
    pub fn project(mut self, projections: ProjectionDefinition) -> Self {
        self.projections.extend(projections);
        self
    }

    /// Adds column selections and aggregations via a builder closure.
    pub fn with_projection_builder<F>(mut self, build: F) -> Self
    where
        F: FnOnce(ProjectionBuilder) -> ProjectionBuilder,
    {
        self.projections.extend(build(ProjectionBuilder::new()).build());
        self
    }

    /// Adds filter predicates (WHERE clause) from a pre-built definition.
    ///
    /// Multiple calls accumulate filters with **implicit AND** logic.
    pub fn filter(mut self, filters: FilterDefinition) -> Self {
        self.filters.extend(filters);
        self
    }

    /// Adds filter predicates via a builder closure.
    pub fn with_filter_builder<F>(mut self, build: F) -> Self
    where
        F: FnOnce(FilterBuilder) -> FilterBuilder,
    {
        self.filters.extend(build(FilterBuilder::new()).build());
        self
    }

    /// Adds sort instructions (ORDER BY clause) from a pre-built definition.
    ///
    /// Multiple calls accumulate sorts — earlier calls take higher sort priority.
    pub fn order_by(mut self, sorts: SortDefinition) -> Self {
        self.sorts.extend(sorts);
        self
    }

    /// Adds sort instructions via a builder closure.
    pub fn with_sort_builder<F>(mut self, build: F) -> Self
    where
        F: FnOnce(SortBuilder) -> SortBuilder,
    {
        self.sorts.extend(build(SortBuilder::new()).build());
        self
    }

    /// Adds group-by fields (GROUP BY clause) from a pre-built definition.
    ///
    /// Multiple calls accumulate group fields.
    pub fn group_by(mut self, groups: GroupDefinition) -> Self {
        self.groups.extend(groups);
        self
    }

    /// Adds group-by fields via a builder closure.
    pub fn with_group_builder<F>(mut self, build: F) -> Self
    where
        F: FnOnce(GroupBuilder) -> GroupBuilder,
    {
        self.groups.extend(build(GroupBuilder::new()).build());
        self
    }

    /// Sets the maximum number of rows to return (LIMIT).
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Sets the number of rows to skip before returning results (OFFSET).
    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }
}
