
use melodium_macro::mel_function;
use melodium_core::*;

/// Return the smallest value that can be represented by `i16`.
/// 
/// The smallest value for `i16` is `-32768`.
#[mel_function]
pub fn min() -> i16 {
    i16::MIN
}

/// Return the largest value that can be represented by `i16`.
/// 
/// The largest value for `i16` is `32767`.
#[mel_function]
pub fn max() -> i16 {
    i16::MAX
}
