mod sort;
mod sort_builder;

use smallvec::SmallVec;

pub use sort::Sort;
pub use sort_builder::SortBuilder;

/// Type alias for a list of sort specifications.
/// Stack-allocated for up to 4 sorts; larger queries spill to heap automatically.
pub type SortDefinition = SmallVec<[Sort; 4]>;
