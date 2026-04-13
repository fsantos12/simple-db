mod projection;
mod projection_builder;

use smallvec::SmallVec;

pub use projection::Projection;
pub use projection_builder::ProjectionBuilder;

/// Type alias for a list of projections (SELECT clauses).
/// Stack-allocated for up to 10 projections; larger queries spill to heap automatically.
pub type ProjectionDefinition = SmallVec<[Projection; 10]>;
