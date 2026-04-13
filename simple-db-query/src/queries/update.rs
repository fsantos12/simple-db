use crate::{builders::{FilterBuilder, FilterDefinition}, types::DbValue};

/// UPDATE query builder for modifying existing data.
///
/// Supports:
/// - Multiple field updates via `.set()` (merged into single UPDATE statement)
/// - WHERE filter conditions to target specific rows
/// - Automatic type conversion via `Into<DbValue>`
///
/// # Example
///
/// ```rust,ignore
/// let query = Query::update("users")
///     .set("email", "newemail@example.com")
///     .set("updated_at", "2024-04-13")
///     .filter(|b| b.eq("id", 42));
/// ```
///
/// # Safety Note
/// 
/// If no filters are specified, the update applies to ALL rows in the collection.
/// Always use `.filter()` unless you explicitly want to update everything.
#[derive(Debug, Clone)]
pub struct UpdateQuery {
    pub collection: String,
    pub updates: Vec<(String, DbValue)>,
    pub filters: FilterDefinition,
}

impl UpdateQuery {
    /// Creates a new update query for the given collection.
    pub fn new<S: Into<String>>(collection: S) -> Self {
        Self {
            collection: collection.into(),
            updates: Vec::new(),
            filters: FilterDefinition::new(),
        }
    }

    /// Sets a field to a new value. Multiple calls accumulate field updates.
    pub fn set<F: Into<String>, V: Into<DbValue>>(mut self, field: F, value: V) -> Self {
        self.updates.push((field.into(), value.into()));
        self
    }

    /// Adds filter conditions (WHERE clause) to target specific rows.
    /// Multiple filter calls use implicit AND logic.
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
}