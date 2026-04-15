//! Shared result type alias for the simple-db ecosystem.
//!
//! All fallible operations across simple-db crates return [`DbResult<T>`],
//! which erases the concrete error type behind `Box<dyn DbError>`. This lets
//! driver and query code compose without depending on each other's error types.

use crate::DbError;

/// Shorthand for `Result<T, Box<dyn DbError>>`.
///
/// Used as the return type for every fallible operation in the simple-db
/// ecosystem. The boxed trait object allows different crates to return their
/// own error types through a single, uniform channel.
pub type DbResult<T> = Result<T, Box<dyn DbError>>;
