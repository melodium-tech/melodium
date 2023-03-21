
use melodium_macro::mel_function;
use melodium_core::*;

/// Return the smallest value that can be represented by `u128`.
/// 
/// The smallest value for `u128` is `0`.
#[mel_function]
pub fn min() -> u128 {
    u128::MIN
}

/// Return the largest value that can be represented by `u128`.
/// 
/// The largest value for `u128` is `340282366920938463463374607431768211455`.
#[mel_function]
pub fn max() -> u128 {
    u128::MAX
}
