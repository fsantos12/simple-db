#[derive(Debug, Clone, PartialEq)]
pub enum Projection {
    // --- Basic ---
    Field(String),
    Alias(String, String),

    // --- Aggregations ---
    CountAll,
    Count(String),
    Sum(String),
    Avg(String),
    Min(String),
    Max(String),
}