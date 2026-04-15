use simple_db_core::DbValue;

use smol_str::SmolStr;

/// A single filter predicate used in WHERE clauses.
///
/// `Filter` variants cover the full range of SQL predicates — null checks,
/// scalar comparisons, pattern matching, range queries, set membership, and
/// logical combinators. Drivers translate these into their native query syntax.
///
/// Filters are produced by [`FilterBuilder`](super::FilterBuilder) and stored
/// in a [`FilterDefinition`](super::FilterDefinition). Multiple top-level
/// filters in a definition are combined with **implicit AND** logic.
#[derive(Debug, Clone)]
pub enum Filter {
    // =========================================================================
    // NULL CHECKS
    // =========================================================================

    /// `field IS NULL` — matches rows where the field has no value.
    IsNull(SmolStr),

    /// `field IS NOT NULL` — matches rows where the field has any value.
    IsNotNull(SmolStr),

    // =========================================================================
    // BASIC COMPARISONS
    // =========================================================================

    /// `field = value` — exact equality.
    Eq(SmolStr, DbValue),

    /// `field != value` — inequality.
    Neq(SmolStr, DbValue),

    /// `field < value` — strictly less than.
    Lt(SmolStr, DbValue),

    /// `field <= value` — less than or equal.
    Lte(SmolStr, DbValue),

    /// `field > value` — strictly greater than.
    Gt(SmolStr, DbValue),

    /// `field >= value` — greater than or equal.
    Gte(SmolStr, DbValue),

    // =========================================================================
    // PATTERN MATCHING  (LIKE / NOT LIKE)
    // =========================================================================

    /// `field LIKE 'value%'` — field starts with the given prefix.
    StartsWith(SmolStr, DbValue),

    /// `field NOT LIKE 'value%'` — field does not start with the given prefix.
    NotStartsWith(SmolStr, DbValue),

    /// `field LIKE '%value'` — field ends with the given suffix.
    EndsWith(SmolStr, DbValue),

    /// `field NOT LIKE '%value'` — field does not end with the given suffix.
    NotEndsWith(SmolStr, DbValue),

    /// `field LIKE '%value%'` — field contains the given substring.
    Contains(SmolStr, DbValue),

    /// `field NOT LIKE '%value%'` — field does not contain the given substring.
    NotContains(SmolStr, DbValue),

    // =========================================================================
    // REGEX MATCHING
    // =========================================================================

    /// `field ~ regex` — field matches the given regular expression pattern.
    Regex(SmolStr, SmolStr),

    // =========================================================================
    // RANGE CHECKS
    // =========================================================================

    /// `field BETWEEN low AND high` — inclusive range check.
    Between(SmolStr, (DbValue, DbValue)),

    /// `field NOT BETWEEN low AND high` — outside of the inclusive range.
    NotBetween(SmolStr, (DbValue, DbValue)),

    // =========================================================================
    // SET MEMBERSHIP
    // =========================================================================

    /// `field IN (v1, v2, ...)` — field value is one of the provided values.
    In(SmolStr, Vec<DbValue>),

    /// `field NOT IN (v1, v2, ...)` — field value is none of the provided values.
    NotIn(SmolStr, Vec<DbValue>),

    // =========================================================================
    // LOGICAL COMBINATORS
    // =========================================================================

    /// `(a AND b AND ...)` — all child predicates must match.
    And(Vec<Filter>),

    /// `(a OR b OR ...)` — at least one child predicate must match.
    Or(Vec<Filter>),

    /// `NOT predicate` — inverts the contained predicate.
    Not(Box<Filter>),
}
