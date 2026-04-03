use crate::types::DbValue;

#[derive(Debug, Clone)]
pub enum Filter {
    // --- Null Checks ---
    IsNull(String),
    IsNotNull(String),

    // --- Basic Comparisons ---
    Eq(String, DbValue),
    Neq(String, DbValue),
    Lt(String, DbValue),
    Lte(String, DbValue),
    Gt(String, DbValue),
    Gte(String, DbValue),

    // --- Pattern Matching ---
    StartsWith(String, DbValue),
    NotStartsWith(String, DbValue),
    EndsWith(String, DbValue),
    NotEndsWith(String, DbValue),
    Contains(String, DbValue),
    NotContains(String, DbValue),

    // --- Regex Matching ---
    Regex(String, String),

    // --- Range Checks ---
    Between(String, DbValue, DbValue),
    NotBetween(String, DbValue, DbValue),

    // --- Set Membership ---
    In(String, Vec<DbValue>),
    NotIn(String, Vec<DbValue>),

    // --- Logical Operators ---
    And(Vec<Filter>),
    Or(Vec<Filter>),
    Not(Box<Filter>)
}