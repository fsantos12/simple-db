use std::error::Error;
use thiserror::Error;

// -----------------------------------------------------------------------------
// 1. O ERRO PRINCIPAL (A Interface Pública)
// -----------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum DbError {
    /// Delega de forma transparente para os erros de Tipagem
    #[error(transparent)]
    Type(#[from] TypeError),

    /// Delega de forma transparente para os erros de Query
    #[error(transparent)]
    Query(#[from] QueryError),

    /// Delega de forma transparente para os erros Nativos do Driver
    #[error(transparent)]
    Driver(#[from] DriverError),

    /// Mantemos o NotFound no topo porque é o erro lógico mais comum num ORM
    #[error("record not found")]
    NotFound,
}

#[derive(Debug, Error)]
pub enum TypeError {
    #[error("type mismatch: expected {expected}, found {found}")]
    Mismatch { expected: String, found: String },

    #[error("index out of bounds: {0}")]
    IndexOutOfBounds(usize),

    #[error("missing column: '{0}'")]
    ColumnMissing(String),
}

#[derive(Debug, Error)]
pub enum QueryError {
    #[error("failed to build query: {0}")]
    Build(String),

    #[error("syntax error: {0}")]
    Syntax(String),

    #[error("invalid argument: {0}")]
    InvalidArgument(String),
}

#[derive(Debug, Error)]
pub enum DriverError {
    #[error("database connection failed")]
    Connection(#[source] Box<dyn Error + Send + Sync + 'static>),

    #[error("query execution failed")]
    Execution(#[source] Box<dyn Error + Send + Sync + 'static>),

    #[error("transaction error")]
    Transaction(#[source] Box<dyn Error + Send + Sync + 'static>),
}