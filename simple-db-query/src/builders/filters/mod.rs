mod filter;
mod filter_builder;

use smallvec::SmallVec;

pub use filter::Filter;
pub use filter_builder::FilterBuilder;

/// Type alias for a list of filter predicates (implicit AND logic).
/// Stack-allocated for up to 8 filters; larger queries spill to heap automatically.
pub type FilterDefinition = SmallVec<[Filter; 8]>;