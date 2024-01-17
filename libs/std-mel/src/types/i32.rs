use melodium_core::*;
use melodium_macro::mel_function;

/// Return the smallest value that can be represented by `i32`.
///
/// The smallest value for `i32` is `-2147483648`.
#[mel_function]
pub fn min() -> i32 {
    i32::MIN
}

/// Return the largest value that can be represented by `i32`.
///
/// The largest value for `i32` is `2147483647`.
#[mel_function]
pub fn max() -> i32 {
    i32::MAX
}
