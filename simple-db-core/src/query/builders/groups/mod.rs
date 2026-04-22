mod group_builder;

use smallvec::SmallVec;
use smol_str::SmolStr;

pub use group_builder::GroupBuilder;

/// Type alias for a list of field names to group by.
/// Stack-allocated for up to 4 group fields; larger queries spill to heap automatically.
pub type GroupDefinition = SmallVec<[SmolStr; 4]>;

#[macro_export]
macro_rules! group {
    // Matches an empty group!()
    () => {
        $crate::query::GroupBuilder::new().build()
    };

    // Matches a comma-separated list of strings: group!("department", "role")
    ( $( $field:expr ),+ $(,)? ) => {
        $crate::query::GroupBuilder::new()
            $( .field($field) )+
            .build()
    };
}