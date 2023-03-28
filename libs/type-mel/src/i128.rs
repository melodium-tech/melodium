use melodium_core::*;
use melodium_macro::mel_function;

/// Return the smallest value that can be represented by `i128`.
///
/// The smallest value for `i128` is `-170141183460469231731687303715884105728`.
#[mel_function]
pub fn min() -> i128 {
    i128::MIN
}

/// Return the largest value that can be represented by `i128`.
///
/// The largest value for `i128` is `170141183460469231731687303715884105727`.
#[mel_function]
pub fn max() -> i128 {
    i128::MAX
}
