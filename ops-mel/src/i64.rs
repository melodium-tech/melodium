use melodium_macro::{check, mel_function, mel_treatment};
use melodium_core::*;

/// Return whether `a` is equal to `b`
#[mel_function]
pub fn equal(a: i64, b: i64) -> bool {
    a == b
}

/// Return whether `a` is different `b`
#[mel_function]
pub fn not_equal(a: i64, b: i64) -> bool {
    a != b
}

/// Determine whether `a` is equal to `b`
#[mel_treatment(
    input a Stream<i64>
    input b Stream<i64>
    output result Stream<bool>
)]
pub async fn equal() {
    while let (Ok(a), Ok(b)) = (a.recv_one_i64().await, b.recv_one_i64().await) {
        check!(result.send_one_bool(a == b).await)
    }
}

/// Determine whether `a` is different from `b`
#[mel_treatment(
    input a Stream<i64>
    input b Stream<i64>
    output result Stream<bool>
)]
pub async fn not_equal() {
    while let (Ok(a), Ok(b)) = (a.recv_one_i64().await, b.recv_one_i64().await) {
        check!(result.send_one_bool(a != b).await)
    }
}

/// Elevates `base` from `exponent`
#[mel_function]
pub fn pow(base: i64, exponent: i64) -> i64 {
    base.pow(exponent as u32)
}

/// Tells whether `a` is greater or equal to `b`.
#[mel_function]
pub fn ge(a: i64, b: i64) -> bool {
    a >= b
}

/// Tells whether `a` is lower or equal to `b`.
#[mel_function]
pub fn le(a: i64, b: i64) -> bool {
    a <= b
}

/// Elevates values from a stream of `i64` to the power of another one.
/// 
/// Values passed through `base` are elevated to the power of `exponent`.
#[mel_treatment(
    input base Stream<i64>
    input exponent Stream<i64>
    output power Stream<i64>
)]
pub async fn pow() {
    while let (Ok(base), Ok(exp)) = (base.recv_one_i64().await, exponent.recv_one_i64().await) {
        check!(power.send_one_i64(base.pow(exp as u32)).await)
    }
}

/// Determine whether `a` is lower or equal to `b`
#[mel_treatment(
    input a Stream<i64>
    input b Stream<i64>
    output is Stream<bool>
)]
pub async fn lower_equal() {
    while let (Ok(a), Ok(b)) = (a.recv_one_i64().await, b.recv_one_i64().await) {
        check!(is.send_one_bool(a <= b).await)
    }
}

/// Determine whether `a` is greater or equal to `b`
#[mel_treatment(
    input a Stream<i64>
    input b Stream<i64>
    output is Stream<bool>
)]
pub async fn greater_equal() {
    while let (Ok(a), Ok(b)) = (a.recv_one_i64().await, b.recv_one_i64().await) {
        check!(is.send_one_bool(a >= b).await)
    }
}


/// Add `a` and `b`
#[mel_function]
pub fn add(a: i64, b: i64) -> i64 {
    a + b
}

/// Divide `dividend` by `divisor`
#[mel_function]
pub fn div(dividend: i64, divisor: i64) -> i64 {
    dividend / divisor
}

/// Multiply `a` by `b`
#[mel_function]
pub fn mult(a: i64, b: i64) -> i64 {
    a * b
}

/// Get the remainder of `dividend` by `divisor`
#[mel_function]
pub fn rem(dividend: i64, divisor: i64) -> i64 {
    dividend % divisor
}

/// Substract `b` from `a`
#[mel_function]
pub fn sub(a: i64, b: i64) -> i64 {
    a - b
}

/// Tells whether `a` is strictly greater than `b`.
#[mel_function]
pub fn gt(a: i64, b: i64) -> bool {
    a > b
}

/// Tells whether `a` is strictly lower than `b`.
#[mel_function]
pub fn lt(a: i64, b: i64) -> bool {
    a < b
}

/// Compares and gives the minimum of two values.
#[mel_function]
pub fn min(a: i64, b: i64) -> i64 {
    a.min(b)
}

/// Compares and gives the maximum of two values.
#[mel_function]
pub fn max(a: i64, b: i64) -> i64 {
    a.max(b)
}

/// Add values from two streams of `i64`.
/// 
/// Values passed through `a` & `b` are added and send in sum.
#[mel_treatment(
    input a Stream<i64>
    input b Stream<i64>
    output sum Stream<i64>
)]
pub async fn add() {
    while let (Ok(a), Ok(b)) = (a.recv_one_i64().await, b.recv_one_i64().await) {
        check!(sum.send_one_i64(a + b).await)
    }
}

/// Divide values from two streams of `i64`.
/// 
/// Every `a` number passed through the stream is divided by `b` counterpart.
#[mel_treatment(
    input a Stream<i64>
    input b Stream<i64>
    output quotient Stream<i64>
)]
pub async fn div() {
    while let (Ok(a), Ok(b)) = (a.recv_one_i64().await, b.recv_one_i64().await) {
        check!(quotient.send_one_i64(a / b).await)
    }
}

/// Multiply values from two streams of `i64`.
/// 
/// Every `a` number passed through the stream is multiplied by `b` counterpart.
#[mel_treatment(
    input a Stream<i64>
    input b Stream<i64>
    output product Stream<i64>
)]
pub async fn mult() {
    while let (Ok(a), Ok(b)) = (a.recv_one_i64().await, b.recv_one_i64().await) {
        check!(product.send_one_i64(a * b).await)
    }
}

/// Give the remainder of the division from two streams of `i64`.
/// 
/// Every `a` number passed through the stream is divided by `b` and the remainder is outputted.
#[mel_treatment(
    input a Stream<i64>
    input b Stream<i64>
    output remainder Stream<i64>
)]
pub async fn rem() {
    while let (Ok(a), Ok(b)) = (a.recv_one_i64().await, b.recv_one_i64().await) {
        check!(remainder.send_one_i64(a % b).await)
    }
}

/// Substract values from two streams of `i64`.
/// 
/// Every `a` number passed through the stream get `b` substracted.
#[mel_treatment(
    input a Stream<i64>
    input b Stream<i64>
    output diff Stream<i64>
)]
pub async fn sub() {
    while let (Ok(a), Ok(b)) = (a.recv_one_i64().await, b.recv_one_i64().await) {
        check!(diff.send_one_i64(a - b).await)
    }
}

/// Compares and gives the minimum of two values.
#[mel_treatment(
    input a Stream<i64>
    input b Stream<i64>
    output min Stream<i64>
)]
pub async fn min() {
    while let (Ok(a), Ok(b)) = (a.recv_one_i64().await, b.recv_one_i64().await) {
        check!(min.send_one_i64(a.min(b)).await)
    }
}

/// Compares and gives the maximum of two values.
#[mel_treatment(
    input a Stream<i64>
    input b Stream<i64>
    output max Stream<i64>
)]
pub async fn max() {
    while let (Ok(a), Ok(b)) = (a.recv_one_i64().await, b.recv_one_i64().await) {
        check!(max.send_one_i64(a.max(b)).await)
    }
}

/// Determine whether `a` is strictly lower than `b`
#[mel_treatment(
    input a Stream<i64>
    input b Stream<i64>
    output is Stream<bool>
)]
pub async fn lower_than() {
    while let (Ok(a), Ok(b)) = (a.recv_one_i64().await, b.recv_one_i64().await) {
        check!(is.send_one_bool(a < b).await)
    }
}

/// Determine whether `a` is strictly greater than `b`
#[mel_treatment(
    input a Stream<i64>
    input b Stream<i64>
    output is Stream<bool>
)]
pub async fn greater_than() {
    while let (Ok(a), Ok(b)) = (a.recv_one_i64().await, b.recv_one_i64().await) {
        check!(is.send_one_bool(a > b).await)
    }
}

/// Get absolute value
#[mel_function]
pub fn abs(value: i64) -> i64 {
    value.abs()
}

/// Get the absolute values from a stream of `i64`.
#[mel_treatment(
    input value Stream<i64>
    output abs Stream<i64>
)]
pub async fn abs() {
    while let Ok(values) = value.recv_i64().await {
        check!(abs.send_i64(values.into_iter().map(|v| v.abs()).collect()).await)
    }
}
