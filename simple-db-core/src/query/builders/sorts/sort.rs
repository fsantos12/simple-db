use smol_str::SmolStr;

/// Sort instruction for result ordering and null value placement.
///
/// The `Sort` enum defines result ordering with support for ascending/descending
/// directions, null placement control, and random ordering for sampling.
#[derive(Debug, Clone)]
pub enum Sort {
    // --- Basic ---

    /// `field ASC` — ascending order (smallest first).
    Asc(SmolStr),

    /// `field DESC` — descending order (largest first).
    Desc(SmolStr),

    // --- Null Handling ---

    /// `field ASC NULLS FIRST` — ascending, with NULL rows at the top.
    AscNullsFirst(SmolStr),

    /// `field ASC NULLS LAST` — ascending, with NULL rows at the bottom.
    AscNullsLast(SmolStr),

    /// `field DESC NULLS FIRST` — descending, with NULL rows at the top.
    DescNullsFirst(SmolStr),

    /// `field DESC NULLS LAST` — descending, with NULL rows at the bottom.
    DescNullsLast(SmolStr),

    // --- Special Cases ---

    /// `ORDER BY RANDOM()` — randomises result order (useful for sampling).
    Random,
}
