#[derive(Debug, Clone, PartialEq)]
pub enum Sort {
    // --- Basic ---
    Asc(String),
    Desc(String),

    // --- Null Handling ---
    AscNullsFirst(String),
    AscNullsLast(String),
    DescNullsFirst(String),
    DescNullsLast(String),

    // --- Special Cases ---
    Random
}