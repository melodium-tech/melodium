use melodium_core::*;
use melodium_macro::mel_function;

/// Return the smallest value that can be represented by `u16`.
///
/// The smallest value for `u16` is `0`.
#[mel_function]
pub fn min() -> u16 {
    u16::MIN
}

/// Return the largest value that can be represented by `u16`.
///
/// The largest value for `u16` is `65535`.
#[mel_function]
pub fn max() -> u16 {
    u16::MAX
}
