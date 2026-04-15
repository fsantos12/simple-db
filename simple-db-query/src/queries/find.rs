use crate::builders::{FilterBuilder, FilterDefinition, GroupBuilder, GroupDefinition, ProjectionBuilder, ProjectionDefinition, SortBuilder, SortDefinition};

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
///     .project(|b| b
///         .field("customer_id")
///         .sum("total")
///         .count_all()
///     )
///     .filter(|b| b.gte("total", 100.0))
///     .filter(|b| b.eq("status", "completed"))
///     .order_by(|b| b.desc("total"))
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

    /// Adds column selections and aggregations (SELECT clause).
    ///
    /// Multiple calls accumulate projections — each call appends to the list
    /// rather than replacing it.
    pub fn project<F>(mut self, build: F) -> Self
    where
        F: FnOnce(ProjectionBuilder) -> ProjectionBuilder,
    {
        self.projections.extend(build(ProjectionBuilder::new()).build());
        self
    }

    /// Adds filter predicates (WHERE clause).
    ///
    /// Multiple calls accumulate filters with **implicit AND** logic.
    pub fn filter<F>(mut self, build: F) -> Self
    where
        F: FnOnce(FilterBuilder) -> FilterBuilder,
    {
        self.filters.extend(build(FilterBuilder::new()).build());
        self
    }

    /// Replaces the filter definition wholesale.
    ///
    /// Useful when passing a pre-built [`FilterDefinition`] from outside the builder.
    pub fn with_filters(mut self, filters: FilterDefinition) -> Self {
        self.filters.extend(filters);
        self
    }

    /// Adds sort instructions (ORDER BY clause).
    ///
    /// Multiple calls accumulate sorts — earlier calls take higher sort priority.
    pub fn order_by<F>(mut self, build: F) -> Self
    where
        F: FnOnce(SortBuilder) -> SortBuilder,
    {
        self.sorts.extend(build(SortBuilder::new()).build());
        self
    }

    /// Adds group-by fields (GROUP BY clause) for aggregate queries.
    ///
    /// Multiple calls accumulate group fields.
    pub fn group_by<F>(mut self, build: F) -> Self
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
