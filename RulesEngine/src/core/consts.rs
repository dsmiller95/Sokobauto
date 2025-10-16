use std::hint::black_box;

pub const DEDUPLICATE_BOXES: bool = true;
pub const TRIM_UNWINNABLE: bool = black_box(true);
pub const TRIM_HEURISTICAL_UNWINNABLE: bool = black_box(true);