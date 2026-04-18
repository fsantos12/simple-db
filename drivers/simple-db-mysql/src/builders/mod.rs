//! SQL clause compilers for the MySQL driver.
//!
//! Each function converts the driver-agnostic builder types from `simple-db-core`
//! into MySQL-specific SQL fragments. Placeholders use `?` positional syntax.

mod projections;
mod filters;
mod sorts;
mod groups;

pub use projections::compile_projections;
pub use filters::compile_filters;
pub use groups::compile_groups;
pub use sorts::compile_sorts;
