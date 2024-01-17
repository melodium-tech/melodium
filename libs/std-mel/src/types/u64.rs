use melodium_core::*;
use melodium_macro::mel_function;

/// Return the smallest value that can be represented by `u64`.
///
/// The smallest value for `u64` is `0`.
#[mel_function]
pub fn min() -> u64 {
    u64::MIN
}

/// Return the largest value that can be represented by `u64`.
///
/// The largest value for `u64` is `18446744073709551615`.
#[mel_function]
pub fn max() -> u64 {
    u64::MAX
}
