
use melodium_macro::mel_function;
use melodium_core::*;

/// Return the smallest value that can be represented by `i64`.
/// 
/// The smallest value for `i64` is `-9223372036854775808`.
#[mel_function]
pub fn min() -> i64 {
    i64::MIN
}

/// Return the largest value that can be represented by `i64`.
/// 
/// The largest value for `i64` is `9223372036854775807`.
#[mel_function]
pub fn max() -> i64 {
    i64::MAX
}
