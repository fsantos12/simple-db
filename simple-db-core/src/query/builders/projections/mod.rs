mod projection;
mod projection_builder;

use smallvec::SmallVec;

pub use projection::Projection;
pub use projection_builder::ProjectionBuilder;

/// Type alias for a list of projections (SELECT clauses).
/// Stack-allocated for up to 10 projections; larger queries spill to heap automatically.
pub type ProjectionDefinition = SmallVec<[Projection; 10]>;

#[macro_export]
macro_rules! macro_name {
    // Matches an empty macro invocation
    () => {
        $crate::query::ProjectionBuilder::new().build()
    };

    // Matches a comma-separated list of builder method calls
    ( $( $method:ident ( $( $arg:expr ),* ) ),+ $(,)? ) => {
        {
            let builder = $crate::query::ProjectionBuilder::new();
            let builder = $( builder.$method( $( $arg ),* ) ).*;
            builder.build() 
        }
    };
}