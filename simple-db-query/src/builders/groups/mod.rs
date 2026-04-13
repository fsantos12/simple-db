mod group_builder;

use smallvec::SmallVec;
use smol_str::SmolStr;

pub use group_builder::GroupBuilder;

/// Type alias for a list of field names to group by.
/// Stack-allocated for up to 4 group fields; larger queries spill to heap automatically.
pub type GroupDefinition = SmallVec<[SmolStr; 4]>;