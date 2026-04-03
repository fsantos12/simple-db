use std::fmt;

#[derive(Debug)]
pub enum DbError {
    /// Database connection failure
    ConnectionError(String),

    /// Syntax or translation error within the Query
    QueryError(String),

    /// Record not found (common in Find operations)
    NotFound,

    /// Type mismatch error (e.g., trying to read a DbValue::Int as a String)
    TypeError(String),

    /// Specific driver error (encapsulates Postgres, Mongo, etc. native errors)
    DriverError(String),

    /// Concurrency or locking issue (e.g., poisoned lock in MemoryDriver)
    ConcurrencyError(String),

    /// Mapping error when converting between DbRow and entity models
    MappingError(String),
}

/// Implementation of Display for user-friendly error messages
impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DbError::ConnectionError(m) => write!(f, "Connection Error: {}", m),
            DbError::QueryError(m) => write!(f, "Query Error: {}", m),
            DbError::NotFound => write!(f, "Record not found"),
            DbError::TypeError(m) => write!(f, "Type Error: {}", m),
            DbError::DriverError(m) => write!(f, "Driver Error: {}", m),
            DbError::ConcurrencyError(m) => write!(f, "Concurrency Error: {}", m),
            DbError::MappingError(m) => write!(f, "Mapping Error: {}", m),
        }
    }
}

/// Standard Error trait implementation for Rust error interoperability
impl std::error::Error for DbError {}