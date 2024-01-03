use melodium_core::*;
use melodium_macro::{check, mel_function, mel_treatment};

/// Return whether `a` is equal to `b`
#[mel_function]
pub fn equal(a: i32, b: i32) -> bool {
    a == b
}

/// Return whether `a` is different `b`
#[mel_function]
pub fn not_equal(a: i32, b: i32) -> bool {
    a != b
}

/// Determine whether `a` is equal to `b`
#[mel_treatment(
    input a Stream<i32>
    input b Stream<i32>
    output result Stream<bool>
)]
pub async fn equal() {
    while let (Ok(a), Ok(b)) = (
        a.recv_one()
            .await
            .map(|val| GetData::<i32>::try_data(val).unwrap()),
        b.recv_one()
            .await
            .map(|val| GetData::<i32>::try_data(val).unwrap()),
    ) {
        check!(result.send_one((a == b).into()).await)
    }
}

/// Determine whether `a` is different from `b`
#[mel_treatment(
    input a Stream<i32>
    input b Stream<i32>
    output result Stream<bool>
)]
pub async fn not_equal() {
    while let (Ok(a), Ok(b)) = (
        a.recv_one()
            .await
            .map(|val| GetData::<i32>::try_data(val).unwrap()),
        b.recv_one()
            .await
            .map(|val| GetData::<i32>::try_data(val).unwrap()),
    ) {
        check!(result.send_one((a != b).into()).await)
    }
}

/// Elevates `base` from `exponent`
#[mel_function]
pub fn pow(base: i32, exponent: i32) -> i32 {
    base.pow(exponent as u32)
}

/// Tells whether `a` is greater or equal to `b`.
#[mel_function]
pub fn ge(a: i32, b: i32) -> bool {
    a >= b
}

/// Tells whether `a` is lower or equal to `b`.
#[mel_function]
pub fn le(a: i32, b: i32) -> bool {
    a <= b
}

/// Elevates values from a stream of `i32` to the power of another one.
///
/// Values passed through `base` are elevated to the power of `exponent`.
#[mel_treatment(
    input base Stream<i32>
    input exponent Stream<i32>
    output power Stream<i32>
)]
pub async fn pow() {
    while let (Ok(base), Ok(exp)) = (
        base.recv_one()
            .await
            .map(|val| GetData::<i32>::try_data(val).unwrap()),
        exponent
            .recv_one()
            .await
            .map(|val| GetData::<i32>::try_data(val).unwrap()),
    ) {
        check!(power.send_one(base.pow(exp as u32).into()).await)
    }
}

/// Determine whether `a` is lower or equal to `b`
#[mel_treatment(
    input a Stream<i32>
    input b Stream<i32>
    output is Stream<bool>
)]
pub async fn lower_equal() {
    while let (Ok(a), Ok(b)) = (
        a.recv_one()
            .await
            .map(|val| GetData::<i32>::try_data(val).unwrap()),
        b.recv_one()
            .await
            .map(|val| GetData::<i32>::try_data(val).unwrap()),
    ) {
        check!(is.send_one((a <= b).into()).await)
    }
}

/// Determine whether `a` is greater or equal to `b`
#[mel_treatment(
    input a Stream<i32>
    input b Stream<i32>
    output is Stream<bool>
)]
pub async fn greater_equal() {
    while let (Ok(a), Ok(b)) = (
        a.recv_one()
            .await
            .map(|val| GetData::<i32>::try_data(val).unwrap()),
        b.recv_one()
            .await
            .map(|val| GetData::<i32>::try_data(val).unwrap()),
    ) {
        check!(is.send_one((a >= b).into()).await)
    }
}

/// Add `a` and `b`
#[mel_function]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// Divide `dividend` by `divisor`
#[mel_function]
pub fn div(dividend: i32, divisor: i32) -> i32 {
    dividend / divisor
}

/// Multiply `a` by `b`
#[mel_function]
pub fn mult(a: i32, b: i32) -> i32 {
    a * b
}

/// Get the remainder of `dividend` by `divisor`
#[mel_function]
pub fn rem(dividend: i32, divisor: i32) -> i32 {
    dividend % divisor
}

/// Substract `b` from `a`
#[mel_function]
pub fn sub(a: i32, b: i32) -> i32 {
    a - b
}

/// Tells whether `a` is strictly greater than `b`.
#[mel_function]
pub fn gt(a: i32, b: i32) -> bool {
    a > b
}

/// Tells whether `a` is strictly lower than `b`.
#[mel_function]
pub fn lt(a: i32, b: i32) -> bool {
    a < b
}

/// Compares and gives the minimum of two values.
#[mel_function]
pub fn min(a: i32, b: i32) -> i32 {
    a.min(b)
}

/// Compares and gives the maximum of two values.
#[mel_function]
pub fn max(a: i32, b: i32) -> i32 {
    a.max(b)
}

/// Add values from two streams of `i32`.
///
/// Values passed through `a` & `b` are added and send in sum.
#[mel_treatment(
    input a Stream<i32>
    input b Stream<i32>
    output sum Stream<i32>
)]
pub async fn add() {
    while let (Ok(a), Ok(b)) = (
        a.recv_one()
            .await
            .map(|val| GetData::<i32>::try_data(val).unwrap()),
        b.recv_one()
            .await
            .map(|val| GetData::<i32>::try_data(val).unwrap()),
    ) {
        check!(sum.send_one((a + b).into()).await)
    }
}

/// Divide values from two streams of `i32`.
///
/// Every `a` number passed through the stream is divided by `b` counterpart.
#[mel_treatment(
    input a Stream<i32>
    input b Stream<i32>
    output quotient Stream<i32>
)]
pub async fn div() {
    while let (Ok(a), Ok(b)) = (
        a.recv_one()
            .await
            .map(|val| GetData::<i32>::try_data(val).unwrap()),
        b.recv_one()
            .await
            .map(|val| GetData::<i32>::try_data(val).unwrap()),
    ) {
        check!(quotient.send_one((a / b).into()).await)
    }
}

/// Multiply values from two streams of `i32`.
///
/// Every `a` number passed through the stream is multiplied by `b` counterpart.
#[mel_treatment(
    input a Stream<i32>
    input b Stream<i32>
    output product Stream<i32>
)]
pub async fn mult() {
    while let (Ok(a), Ok(b)) = (
        a.recv_one()
            .await
            .map(|val| GetData::<i32>::try_data(val).unwrap()),
        b.recv_one()
            .await
            .map(|val| GetData::<i32>::try_data(val).unwrap()),
    ) {
        check!(product.send_one((a * b).into()).await)
    }
}

/// Give the remainder of the division from two streams of `i32`.
///
/// Every `a` number passed through the stream is divided by `b` and the remainder is outputted.
#[mel_treatment(
    input a Stream<i32>
    input b Stream<i32>
    output remainder Stream<i32>
)]
pub async fn rem() {
    while let (Ok(a), Ok(b)) = (
        a.recv_one()
            .await
            .map(|val| GetData::<i32>::try_data(val).unwrap()),
        b.recv_one()
            .await
            .map(|val| GetData::<i32>::try_data(val).unwrap()),
    ) {
        check!(remainder.send_one((a % b).into()).await)
    }
}

/// Substract values from two streams of `i32`.
///
/// Every `a` number passed through the stream get `b` substracted.
#[mel_treatment(
    input a Stream<i32>
    input b Stream<i32>
    output diff Stream<i32>
)]
pub async fn sub() {
    while let (Ok(a), Ok(b)) = (
        a.recv_one()
            .await
            .map(|val| GetData::<i32>::try_data(val).unwrap()),
        b.recv_one()
            .await
            .map(|val| GetData::<i32>::try_data(val).unwrap()),
    ) {
        check!(diff.send_one((a - b).into()).await)
    }
}

/// Compares and gives the minimum of two values.
#[mel_treatment(
    input a Stream<i32>
    input b Stream<i32>
    output min Stream<i32>
)]
pub async fn min() {
    while let (Ok(a), Ok(b)) = (
        a.recv_one()
            .await
            .map(|val| GetData::<i32>::try_data(val).unwrap()),
        b.recv_one()
            .await
            .map(|val| GetData::<i32>::try_data(val).unwrap()),
    ) {
        check!(min.send_one(a.min(b).into()).await)
    }
}

/// Compares and gives the maximum of two values.
#[mel_treatment(
    input a Stream<i32>
    input b Stream<i32>
    output max Stream<i32>
)]
pub async fn max() {
    while let (Ok(a), Ok(b)) = (
        a.recv_one()
            .await
            .map(|val| GetData::<i32>::try_data(val).unwrap()),
        b.recv_one()
            .await
            .map(|val| GetData::<i32>::try_data(val).unwrap()),
    ) {
        check!(max.send_one(a.max(b).into()).await)
    }
}

/// Determine whether `a` is strictly lower than `b`
#[mel_treatment(
    input a Stream<i32>
    input b Stream<i32>
    output is Stream<bool>
)]
pub async fn lower_than() {
    while let (Ok(a), Ok(b)) = (
        a.recv_one()
            .await
            .map(|val| GetData::<i32>::try_data(val).unwrap()),
        b.recv_one()
            .await
            .map(|val| GetData::<i32>::try_data(val).unwrap()),
    ) {
        check!(is.send_one((a < b).into()).await)
    }
}

/// Determine whether `a` is strictly greater than `b`
#[mel_treatment(
    input a Stream<i32>
    input b Stream<i32>
    output is Stream<bool>
)]
pub async fn greater_than() {
    while let (Ok(a), Ok(b)) = (
        a.recv_one()
            .await
            .map(|val| GetData::<i32>::try_data(val).unwrap()),
        b.recv_one()
            .await
            .map(|val| GetData::<i32>::try_data(val).unwrap()),
    ) {
        check!(is.send_one((a > b).into()).await)
    }
}

/// Get absolute value
#[mel_function]
pub fn abs(value: i32) -> i32 {
    value.abs()
}

/// Get the absolute values from a stream of `i32`.
#[mel_treatment(
    input value Stream<i32>
    output abs Stream<i32>
)]
pub async fn abs() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| TryInto::<Vec<i32>>::try_into(values).unwrap())
    {
        check!(
            abs.send_many(
                values
                    .into_iter()
                    .map(|v| v.abs())
                    .collect::<VecDeque<_>>()
                    .into()
            )
            .await
        )
    }
}
