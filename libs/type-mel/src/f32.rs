
/// Return the positive infinity for `f32`.
#[mel_function]
pub fn infinity() -> f32 {
    f32::INFINITY
}

/// Return the negative infinity for `f32`.
#[mel_function]
pub fn neg_infinity() -> f32 {
    f32::NEG_INFINITY
}

/// Return the not-a-number value for `f32`.
#[mel_function]
pub fn nan() -> f32 {
    f32::NAN
}

use melodium_macro::mel_function;
use melodium_core::*;

/// Return the smallest value that can be represented by `f32`.
/// 
/// The smallest value for `f32` is `-340282350000000000000000000000000000000`.
#[mel_function]
pub fn min() -> f32 {
    f32::MIN
}

/// Return the largest value that can be represented by `f32`.
/// 
/// The largest value for `f32` is `340282350000000000000000000000000000000`.
#[mel_function]
pub fn max() -> f32 {
    f32::MAX
}
