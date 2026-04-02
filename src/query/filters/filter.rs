use crate::value::DbValue;

#[derive(Debug, Clone, PartialEq)]
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
    Like(String, String),
    NotLike(String, String),

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