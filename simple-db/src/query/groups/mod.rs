//! GROUP BY clause construction and aggregation grouping.
//!
//! Provides `GroupBuilder` for fluent construction of GROUP BY clauses used in
//! aggregate queries to partition results by one or more fields.

use smallvec::SmallVec;
use smol_str::SmolStr;

mod group_builder;

/// Type alias for a list of field names to group by.
/// Stack-allocated for up to 4 group fields; larger queries spill to heap automatically.
pub type GroupDefinition = SmallVec<[SmolStr; 4]>;

pub use group_builder::GroupBuilder;