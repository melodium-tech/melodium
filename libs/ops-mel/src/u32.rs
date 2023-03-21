use melodium_macro::{check, mel_function, mel_treatment};
use melodium_core::*;

/// Return whether `a` is equal to `b`
#[mel_function]
pub fn equal(a: u32, b: u32) -> bool {
    a == b
}

/// Return whether `a` is different `b`
#[mel_function]
pub fn not_equal(a: u32, b: u32) -> bool {
    a != b
}

/// Determine whether `a` is equal to `b`
#[mel_treatment(
    input a Stream<u32>
    input b Stream<u32>
    output result Stream<bool>
)]
pub async fn equal() {
    while let (Ok(a), Ok(b)) = (a.recv_one_u32().await, b.recv_one_u32().await) {
        check!(result.send_one_bool(a == b).await)
    }
}

/// Determine whether `a` is different from `b`
#[mel_treatment(
    input a Stream<u32>
    input b Stream<u32>
    output result Stream<bool>
)]
pub async fn not_equal() {
    while let (Ok(a), Ok(b)) = (a.recv_one_u32().await, b.recv_one_u32().await) {
        check!(result.send_one_bool(a != b).await)
    }
}

/// Elevates `base` from `exponent`
#[mel_function]
pub fn pow(base: u32, exponent: u32) -> u32 {
    base.pow(exponent as u32)
}

/// Tells whether `a` is greater or equal to `b`.
#[mel_function]
pub fn ge(a: u32, b: u32) -> bool {
    a >= b
}

/// Tells whether `a` is lower or equal to `b`.
#[mel_function]
pub fn le(a: u32, b: u32) -> bool {
    a <= b
}

/// Elevates values from a stream of `u32` to the power of another one.
/// 
/// Values passed through `base` are elevated to the power of `exponent`.
#[mel_treatment(
    input base Stream<u32>
    input exponent Stream<u32>
    output power Stream<u32>
)]
pub async fn pow() {
    while let (Ok(base), Ok(exp)) = (base.recv_one_u32().await, exponent.recv_one_u32().await) {
        check!(power.send_one_u32(base.pow(exp as u32)).await)
    }
}

/// Determine whether `a` is lower or equal to `b`
#[mel_treatment(
    input a Stream<u32>
    input b Stream<u32>
    output is Stream<bool>
)]
pub async fn lower_equal() {
    while let (Ok(a), Ok(b)) = (a.recv_one_u32().await, b.recv_one_u32().await) {
        check!(is.send_one_bool(a <= b).await)
    }
}

/// Determine whether `a` is greater or equal to `b`
#[mel_treatment(
    input a Stream<u32>
    input b Stream<u32>
    output is Stream<bool>
)]
pub async fn greater_equal() {
    while let (Ok(a), Ok(b)) = (a.recv_one_u32().await, b.recv_one_u32().await) {
        check!(is.send_one_bool(a >= b).await)
    }
}


/// Add `a` and `b`
#[mel_function]
pub fn add(a: u32, b: u32) -> u32 {
    a + b
}

/// Divide `dividend` by `divisor`
#[mel_function]
pub fn div(dividend: u32, divisor: u32) -> u32 {
    dividend / divisor
}

/// Multiply `a` by `b`
#[mel_function]
pub fn mult(a: u32, b: u32) -> u32 {
    a * b
}

/// Get the remainder of `dividend` by `divisor`
#[mel_function]
pub fn rem(dividend: u32, divisor: u32) -> u32 {
    dividend % divisor
}

/// Substract `b` from `a`
#[mel_function]
pub fn sub(a: u32, b: u32) -> u32 {
    a - b
}

/// Tells whether `a` is strictly greater than `b`.
#[mel_function]
pub fn gt(a: u32, b: u32) -> bool {
    a > b
}

/// Tells whether `a` is strictly lower than `b`.
#[mel_function]
pub fn lt(a: u32, b: u32) -> bool {
    a < b
}

/// Compares and gives the minimum of two values.
#[mel_function]
pub fn min(a: u32, b: u32) -> u32 {
    a.min(b)
}

/// Compares and gives the maximum of two values.
#[mel_function]
pub fn max(a: u32, b: u32) -> u32 {
    a.max(b)
}

/// Add values from two streams of `u32`.
/// 
/// Values passed through `a` & `b` are added and send in sum.
#[mel_treatment(
    input a Stream<u32>
    input b Stream<u32>
    output sum Stream<u32>
)]
pub async fn add() {
    while let (Ok(a), Ok(b)) = (a.recv_one_u32().await, b.recv_one_u32().await) {
        check!(sum.send_one_u32(a + b).await)
    }
}

/// Divide values from two streams of `u32`.
/// 
/// Every `a` number passed through the stream is divided by `b` counterpart.
#[mel_treatment(
    input a Stream<u32>
    input b Stream<u32>
    output quotient Stream<u32>
)]
pub async fn div() {
    while let (Ok(a), Ok(b)) = (a.recv_one_u32().await, b.recv_one_u32().await) {
        check!(quotient.send_one_u32(a / b).await)
    }
}

/// Multiply values from two streams of `u32`.
/// 
/// Every `a` number passed through the stream is multiplied by `b` counterpart.
#[mel_treatment(
    input a Stream<u32>
    input b Stream<u32>
    output product Stream<u32>
)]
pub async fn mult() {
    while let (Ok(a), Ok(b)) = (a.recv_one_u32().await, b.recv_one_u32().await) {
        check!(product.send_one_u32(a * b).await)
    }
}

/// Give the remainder of the division from two streams of `u32`.
/// 
/// Every `a` number passed through the stream is divided by `b` and the remainder is outputted.
#[mel_treatment(
    input a Stream<u32>
    input b Stream<u32>
    output remainder Stream<u32>
)]
pub async fn rem() {
    while let (Ok(a), Ok(b)) = (a.recv_one_u32().await, b.recv_one_u32().await) {
        check!(remainder.send_one_u32(a % b).await)
    }
}

/// Substract values from two streams of `u32`.
/// 
/// Every `a` number passed through the stream get `b` substracted.
#[mel_treatment(
    input a Stream<u32>
    input b Stream<u32>
    output diff Stream<u32>
)]
pub async fn sub() {
    while let (Ok(a), Ok(b)) = (a.recv_one_u32().await, b.recv_one_u32().await) {
        check!(diff.send_one_u32(a - b).await)
    }
}

/// Compares and gives the minimum of two values.
#[mel_treatment(
    input a Stream<u32>
    input b Stream<u32>
    output min Stream<u32>
)]
pub async fn min() {
    while let (Ok(a), Ok(b)) = (a.recv_one_u32().await, b.recv_one_u32().await) {
        check!(min.send_one_u32(a.min(b)).await)
    }
}

/// Compares and gives the maximum of two values.
#[mel_treatment(
    input a Stream<u32>
    input b Stream<u32>
    output max Stream<u32>
)]
pub async fn max() {
    while let (Ok(a), Ok(b)) = (a.recv_one_u32().await, b.recv_one_u32().await) {
        check!(max.send_one_u32(a.max(b)).await)
    }
}

/// Determine whether `a` is strictly lower than `b`
#[mel_treatment(
    input a Stream<u32>
    input b Stream<u32>
    output is Stream<bool>
)]
pub async fn lower_than() {
    while let (Ok(a), Ok(b)) = (a.recv_one_u32().await, b.recv_one_u32().await) {
        check!(is.send_one_bool(a < b).await)
    }
}

/// Determine whether `a` is strictly greater than `b`
#[mel_treatment(
    input a Stream<u32>
    input b Stream<u32>
    output is Stream<bool>
)]
pub async fn greater_than() {
    while let (Ok(a), Ok(b)) = (a.recv_one_u32().await, b.recv_one_u32().await) {
        check!(is.send_one_bool(a > b).await)
    }
}
