
use melodium_macro::mel_function;
use melodium_core::*;

/// Return the smallest value that can be represented by `byte`.
/// 
/// The smallest value for `byte` is `0x00`.
#[mel_function]
pub fn min() -> byte {
    byte::MIN
}

/// Return the largest value that can be represented by `byte`.
/// 
/// The largest value for `byte` is `0xFF`.
#[mel_function]
pub fn max() -> byte {
    byte::MAX
}
