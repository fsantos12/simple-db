//! SQL clause compilers for the PostgreSQL driver.
//!
//! Each function converts the driver-agnostic builder types from `simple-db-core`
//! into PostgreSQL-specific SQL fragments. Placeholders use `$N` positional syntax.

mod projections;
mod filters;
mod sorts;
mod groups;

pub use projections::compile_projections;
pub use filters::{compile_filters, compile_filters_with_offset};
pub use groups::compile_groups;
pub use sorts::compile_sorts;
