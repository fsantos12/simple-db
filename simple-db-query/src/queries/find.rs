use crate::builders::{FilterBuilder, FilterDefinition, GroupBuilder, GroupDefinition, ProjectionBuilder, ProjectionDefinition, SortBuilder, SortDefinition};

/// SELECT query builder for reading data from a collection.
///
/// Supports all SQL SELECT capabilities:
/// - Column selection (projections) with aggregates
/// - Filtering (WHERE clause)
/// - Sorting (ORDER BY)
/// - Grouping (GROUP BY aggregates)
/// - Pagination (LIMIT/OFFSET)
///
/// # Example
///
/// ```rust,ignore
/// let query = Query::find("orders")
///     .project(|b| b
///         .field("customer_id")
///         .sum("total").as_("grand_total")
///         .count_all().as_("order_count")
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
    /// Creates a new find query for the given collection.
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

    /// Adds column selections and aggregations using a builder closure.
    /// Multiple calls replace previous projections.
    pub fn project<F>(mut self, build: F) -> Self 
    where F: FnOnce(ProjectionBuilder) -> ProjectionBuilder {
        let builder = build(ProjectionBuilder::new());
        self.projections.extend(builder.build());
        self
    }

    /// Adds filter conditions (WHERE clause). Multiple filter calls use implicit AND logic.
    pub fn filter<F>(mut self, build: F) -> Self 
    where F: FnOnce(FilterBuilder) -> FilterBuilder {
        let builder = build(FilterBuilder::new());
        self.filters.extend(builder.build());
        self
    }

    /// Sets the entire filter definition at once.
    pub fn with_filters(mut self, filters: FilterDefinition) -> Self {
        self.filters.extend(filters);
        self
    }

    /// Adds sorting specifications (ORDER BY clause).
    pub fn order_by<F>(mut self, build: F) -> Self 
    where F: FnOnce(SortBuilder) -> SortBuilder {
        let builder = build(SortBuilder::new());
        self.sorts.extend(builder.build());
        self
    }

    /// Adds grouping specifications (GROUP BY clause) for aggregate queries.
    pub fn group_by<F>(mut self, build: F) -> Self 
    where F: FnOnce(GroupBuilder) -> GroupBuilder {
        let builder = build(GroupBuilder::new());
        self.groups.extend(builder.build());
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