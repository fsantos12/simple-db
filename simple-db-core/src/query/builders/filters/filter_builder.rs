use smallvec::SmallVec;
use smol_str::SmolStr;

use crate::{query::builders::{FilterDefinition, filters::Filter}, types::DbValue};

/// Fluent builder for composing filter predicates (WHERE conditions).
///
/// `FilterBuilder` constructs a list of [`Filter`] predicates using method chaining.
/// Multiple predicates added to the same builder are combined with **implicit AND** logic.
/// For explicit OR/AND/NOT grouping use the [`.and()`](Self::and), [`.or()`](Self::or),
/// and [`.not()`](Self::not) methods.
///
/// # Examples
///
/// ```rust
/// use simple_db_core::{FilterBuilder, Filter};
///
/// // Simple equality filter
/// let filters = FilterBuilder::new()
///     .eq("status", "active")
///     .gte("age", 18i32)
///     .build();
///
/// assert_eq!(filters.len(), 2);
///
/// // Logical grouping
/// let filters = FilterBuilder::new()
///     .or(|b| b.eq("role", "admin").eq("role", "moderator"))
///     .build();
///
/// assert_eq!(filters.len(), 1);
/// ```
pub struct FilterBuilder(FilterDefinition);

impl FilterBuilder {
    /// Creates a new empty `FilterBuilder`.
    pub fn new() -> Self {
        Self(SmallVec::new())
    }

    /// Appends a single [`Filter`] predicate to this builder.
    pub fn add(mut self, filter: Filter) -> Self {
        self.0.push(filter);
        self
    }

    /// Appends all filters from an iterator to this builder.
    pub fn extend<I>(mut self, filters: I) -> Self
    where I: IntoIterator<Item = Filter> {
        self.0.extend(filters);
        self
    }

    // =========================================================================
    // NULL CHECKS
    // =========================================================================

    /// Adds an `IS NULL` check on `field`.
    pub fn is_null<F: Into<SmolStr>>(self, field: F) -> Self {
        self.add(Filter::IsNull(field.into()))
    }

    /// Adds an `IS NOT NULL` check on `field`.
    pub fn is_not_null<F: Into<SmolStr>>(self, field: F) -> Self {
        self.add(Filter::IsNotNull(field.into()))
    }

    // =========================================================================
    // BASIC COMPARISONS
    // =========================================================================

    /// Adds an equality filter: `field = value`.
    pub fn eq<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::Eq(field.into(), value.into()))
    }

    /// Adds an inequality filter: `field != value`.
    pub fn neq<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::Neq(field.into(), value.into()))
    }

    /// Adds a less-than filter: `field < value`.
    pub fn lt<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::Lt(field.into(), value.into()))
    }

    /// Adds a less-than-or-equal filter: `field <= value`.
    pub fn lte<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::Lte(field.into(), value.into()))
    }

    /// Adds a greater-than filter: `field > value`.
    pub fn gt<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::Gt(field.into(), value.into()))
    }

    /// Adds a greater-than-or-equal filter: `field >= value`.
    pub fn gte<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::Gte(field.into(), value.into()))
    }

    // =========================================================================
    // PATTERN MATCHING
    // =========================================================================

    /// Adds a prefix match: `field LIKE 'value%'`.
    pub fn starts_with<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::StartsWith(field.into(), value.into()))
    }

    /// Adds a negated prefix match: `field NOT LIKE 'value%'`.
    pub fn not_starts_with<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::NotStartsWith(field.into(), value.into()))
    }

    /// Adds a substring match: `field LIKE '%value%'`.
    pub fn contains<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::Contains(field.into(), value.into()))
    }

    /// Adds a negated substring match: `field NOT LIKE '%value%'`.
    pub fn not_contains<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::NotContains(field.into(), value.into()))
    }

    /// Adds a suffix match: `field LIKE '%value'`.
    pub fn ends_with<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::EndsWith(field.into(), value.into()))
    }

    /// Adds a negated suffix match: `field NOT LIKE '%value'`.
    pub fn not_ends_with<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::NotEndsWith(field.into(), value.into()))
    }

    /// Adds a regular expression match on `field`.
    pub fn regex<F: Into<SmolStr>, R: Into<SmolStr>>(self, field: F, regex: R) -> Self {
        self.add(Filter::Regex(field.into(), regex.into()))
    }

    // =========================================================================
    // RANGE CHECKS
    // =========================================================================

    /// Adds a range filter: `field BETWEEN low AND high` (inclusive).
    pub fn between<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, low: V, high: V) -> Self {
        self.add(Filter::Between(field.into(), (low.into(), high.into())))
    }

    /// Adds a negated range filter: `field NOT BETWEEN low AND high`.
    pub fn not_between<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, low: V, high: V) -> Self {
        self.add(Filter::NotBetween(field.into(), (low.into(), high.into())))
    }

    // =========================================================================
    // SET MEMBERSHIP
    // =========================================================================

    /// Adds a set membership filter: `field IN (values)`.
    pub fn is_in<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, values: Vec<V>) -> Self {
        let db_values: Vec<DbValue> = values.into_iter().map(Into::into).collect();
        self.add(Filter::In(field.into(), db_values))
    }

    /// Adds a negated set membership filter: `field NOT IN (values)`.
    pub fn not_in<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, values: Vec<V>) -> Self {
        let db_values: Vec<DbValue> = values.into_iter().map(Into::into).collect();
        self.add(Filter::NotIn(field.into(), db_values))
    }

    // =========================================================================
    // LOGICAL GROUPING
    // =========================================================================

    /// Groups a set of predicates with AND: `(a AND b AND ...)`.
    ///
    /// If the closure produces no predicates the group is silently dropped.
    pub fn and<I>(self, filters: I) -> Self
    where I: IntoIterator<Item = Filter> {
        let vec: Vec<Filter> = filters.into_iter().collect();
        if vec.is_empty() {
            self
        } else {
            self.add(Filter::And(vec))
        }
    }

    /// Groups a set of predicates with OR: `(a OR b OR ...)`.
    ///
    /// If the closure produces no predicates the group is silently dropped.
    pub fn or<I>(self, filters: I) -> Self
    where I: IntoIterator<Item = Filter> {
        let vec: Vec<Filter> = filters.into_iter().collect();
        if vec.is_empty() {
            self
        } else {
            self.add(Filter::Or(vec))
        }
    }

    /// Negates a predicate or group: `NOT (...)`.
    ///
    /// - Zero predicates → silently dropped.
    /// - One predicate → `NOT predicate`.
    /// - Two or more → `NOT (a AND b AND ...)`.
    pub fn not<I>(self, filters: I) -> Self
    where I: IntoIterator<Item = Filter> {
        let mut vec: Vec<Filter> = filters.into_iter().collect();
        match vec.len() {
            0 => self,
            1 => self.add(Filter::Not(Box::new(vec.pop().unwrap()))),
            _ => self.add(Filter::Not(Box::new(Filter::And(vec)))),
        }
    }

    // =========================================================================
    // FINALIZATION
    // =========================================================================

    /// Consumes the builder and returns the collected [`Filter`] predicates.
    pub fn build(self) -> SmallVec<[Filter; 8]> {
        self.0
    }
}

impl Default for FilterBuilder {
    fn default() -> Self {
        Self::new()
    }
}
