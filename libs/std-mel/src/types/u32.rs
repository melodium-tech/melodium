use melodium_core::*;
use melodium_macro::mel_function;

/// Return the smallest value that can be represented by `u32`.
///
/// The smallest value for `u32` is `0`.
#[mel_function]
pub fn min() -> u32 {
    u32::MIN
}

/// Return the largest value that can be represented by `u32`.
///
/// The largest value for `u32` is `4294967295`.
#[mel_function]
pub fn max() -> u32 {
    u32::MAX
}
