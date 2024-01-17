use melodium_core::*;
use melodium_macro::mel_function;

/// Return the smallest value that can be represented by `i8`.
///
/// The smallest value for `i8` is `-128`.
#[mel_function]
pub fn min() -> i8 {
    i8::MIN
}

/// Return the largest value that can be represented by `i8`.
///
/// The largest value for `i8` is `127`.
#[mel_function]
pub fn max() -> i8 {
    i8::MAX
}
