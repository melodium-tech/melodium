
use melodium_macro::mel_function;
use melodium_core::*;

/// Return the smallest value that can be represented by `u8`.
/// 
/// The smallest value for `u8` is `0`.
#[mel_function]
pub fn min() -> u8 {
    u8::MIN
}

/// Return the largest value that can be represented by `u8`.
/// 
/// The largest value for `u8` is `255`.
#[mel_function]
pub fn max() -> u8 {
    u8::MAX
}
