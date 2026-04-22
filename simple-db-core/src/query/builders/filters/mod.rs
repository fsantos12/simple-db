mod filter;
mod filter_builder;

use smallvec::SmallVec;

pub use filter::Filter;
pub use filter_builder::FilterBuilder;

/// Type alias for a list of filter predicates (implicit AND logic).
/// Stack-allocated for up to 8 filters; larger queries spill to heap automatically.
pub type FilterDefinition = SmallVec<[Filter; 8]>;

#[macro_export]
macro_rules! filter {
    // Matches an empty filter!()
    () => {
        $crate::query::FilterBuilder::new().build()
    };

    // Matches a comma-separated list of builder method calls
    ( $( $method:ident ( $( $arg:expr ),* ) ),+ $(,)? ) => {
        {
            let builder = $crate::query::FilterBuilder::new();
            let builder = $( builder.$method( $( $arg ),* ) ).*;
            builder.build() // Returns the SmallVec<[Filter; 8]>
        }
    };
}