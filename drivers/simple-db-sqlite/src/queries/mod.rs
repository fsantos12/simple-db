//! SQL query compilers for the SQLite driver.
//!
//! Each function takes a typed query object and returns a `(sql, parameters)` pair
//! ready to be executed via sqlx. Placeholders use `?` positional syntax.

mod find;
mod insert;
mod update;
mod delete;

pub use find::compile_find_query;
pub use insert::compile_insert_query;
pub use update::compile_update_query;
pub use delete::compile_delete_query;