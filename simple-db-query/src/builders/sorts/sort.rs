use smol_str::SmolStr;

/// Sort instruction for result ordering and null value placement.
///
/// The `Sort` enum defines result ordering with support for ascending/descending
/// directions, null placement control, and random ordering for sampling.
#[derive(Debug, Clone)]
pub enum Sort {
    // --- Basic ---
    Asc(SmolStr),
    Desc(SmolStr),

    // --- Null Handling ---
    AscNullsFirst(SmolStr),
    AscNullsLast(SmolStr),
    DescNullsFirst(SmolStr),
    DescNullsLast(SmolStr),

    // --- Special Cases ---
    Random,
}
