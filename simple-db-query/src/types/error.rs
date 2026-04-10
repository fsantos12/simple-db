use std::error::Error;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    /// Database connection failure
    #[error("connection error: {0}")]
    ConnectionError(String),

    /// Syntax or translation error within the Query
    #[error("query error: {0}")]
    QueryError(String),

    /// Record not found (common in Find operations)
    #[error("record not found")]
    NotFound,

    /// Type mismatch error with structured expected/found details
    #[error("type mismatch: expected {expected}, found {found}")]
    TypeError { expected: String, found: String },

    /// Encapsulates native errors from specific drivers (Postgres, Mongo, etc.)
    #[error("underlying driver error")]
    DriverError(#[source] Box<dyn Error + Send + Sync + 'static>),
}