
/// Return the positive infinity for `f64`.
#[mel_function]
pub fn infinity() -> f64 {
    f64::INFINITY
}

/// Return the negative infinity for `f64`.
#[mel_function]
pub fn neg_infinity() -> f64 {
    f64::NEG_INFINITY
}

/// Return the not-a-number value for `f64`.
#[mel_function]
pub fn nan() -> f64 {
    f64::NAN
}

use melodium_macro::mel_function;
use melodium_core::*;

/// Return the smallest value that can be represented by `f64`.
/// 
/// The smallest value for `f64` is `-179769313486231570000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000`.
#[mel_function]
pub fn min() -> f64 {
    f64::MIN
}

/// Return the largest value that can be represented by `f64`.
/// 
/// The largest value for `f64` is `179769313486231570000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000`.
#[mel_function]
pub fn max() -> f64 {
    f64::MAX
}
