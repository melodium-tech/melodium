use melodium_core::*;
use melodium_macro::{check, mel_function, mel_treatment};

/// Elevates `base` from `exponent`
#[mel_function]
pub fn pow(base: f32, exponent: f32) -> f32 {
    base.powf(exponent)
}

/// Computes cube root of `value`
#[mel_function]
pub fn cbrt(value: f32) -> f32 {
    value.cbrt()
}

/// Computes natural logarithm of `value`
#[mel_function]
pub fn ln(value: f32) -> f32 {
    value.ln()
}

/// Computes logarithm of `value` with `base`
#[mel_function]
pub fn log(value: f32, base: f32) -> f32 {
    value.log(base)
}

/// Computes quare root of `value`
#[mel_function]
pub fn sqrt(value: f32) -> f32 {
    value.sqrt()
}

/// Computes arccosine of `value` (in radians)
#[mel_function]
pub fn acos(value: f32) -> f32 {
    value.acos()
}

/// Computes inverse hyperbolic cosine of `value`
#[mel_function]
pub fn acosh(value: f32) -> f32 {
    value.acosh()
}

/// Computes arcsine of `value` (in radians)
#[mel_function]
pub fn asin(value: f32) -> f32 {
    value.asin()
}

/// Computes inverse hyperbolic sine of `value`
#[mel_function]
pub fn asinh(value: f32) -> f32 {
    value.asinh()
}

/// Computes arctangent of `value` (in radians)
#[mel_function]
pub fn atan(value: f32) -> f32 {
    value.atan()
}

/// Computes inverse hyperbolic tangent of `value`
#[mel_function]
pub fn atanh(value: f32) -> f32 {
    value.atanh()
}

/// Computes cosine of `value` (in radians)
#[mel_function]
pub fn cos(value: f32) -> f32 {
    value.cos()
}

/// Computes hyperbolic cosine of `value`
#[mel_function]
pub fn cosh(value: f32) -> f32 {
    value.cosh()
}

/// Computes sine of `value` (in radians)
#[mel_function]
pub fn sin(value: f32) -> f32 {
    value.sin()
}

/// Computes hyperbolic sine of `value`
#[mel_function]
pub fn sinh(value: f32) -> f32 {
    value.sinh()
}

/// Computes tangent of `value` (in radians)
#[mel_function]
pub fn tan(value: f32) -> f32 {
    value.tan()
}

/// Computes hyperbolic tangent of `value`
#[mel_function]
pub fn tanh(value: f32) -> f32 {
    value.tanh()
}

/// Elevates values from a stream of `f32` to the power of another one.
///
/// Values passed through `base` are elevated to the power of `exponent`.
#[mel_treatment(
    input base Stream<f32>
    input exponent Stream<f32>
    output power Stream<f32>
)]
pub async fn pow() {
    while let (Ok(base), Ok(exp)) = (
        base.recv_one()
            .await
            .map(|val| GetData::<f32>::try_data(val).unwrap()),
        exponent
            .recv_one()
            .await
            .map(|val| GetData::<f32>::try_data(val).unwrap()),
    ) {
        check!(power.send_one(base.powf(exp).into()).await)
    }
}

/// Computes the cube roots from a stream of `f32`.
#[mel_treatment(
    input value Stream<f32>
    output root Stream<f32>
)]
pub async fn cbrt() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| TryInto::<Vec<f32>>::try_into(values).unwrap())
    {
        check!(
            root.send_many(
                values
                    .into_iter()
                    .map(|v| v.cbrt())
                    .collect::<VecDeque<_>>()
                    .into()
            )
            .await
        )
    }
}

/// Computes the natural logarithms of a stream of `f32`.
#[mel_treatment(
    input value Stream<f32>
    output log Stream<f32>
)]
pub async fn ln() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| TryInto::<Vec<f32>>::try_into(values).unwrap())
    {
        check!(
            log.send_many(
                values
                    .into_iter()
                    .map(|v| v.ln())
                    .collect::<VecDeque<_>>()
                    .into()
            )
            .await
        )
    }
}

/// Computes logarithms from a stream of `f32` with the base of another one.
#[mel_treatment(
    input base Stream<f32>
    input value Stream<f32>
    output log Stream<f32>
)]
pub async fn log() {
    while let (Ok(base), Ok(value)) = (
        base.recv_one()
            .await
            .map(|val| GetData::<f32>::try_data(val).unwrap()),
        value
            .recv_one()
            .await
            .map(|val| GetData::<f32>::try_data(val).unwrap()),
    ) {
        check!(log.send_one(value.log(base).into()).await)
    }
}

/// Computes the square roots from a stream of `f32`.
#[mel_treatment(
    input value Stream<f32>
    output root Stream<f32>
)]
pub async fn sqrt() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| TryInto::<Vec<f32>>::try_into(values).unwrap())
    {
        check!(
            root.send_many(
                values
                    .into_iter()
                    .map(|v| v.sqrt())
                    .collect::<VecDeque<_>>()
                    .into()
            )
            .await
        )
    }
}

/// Computes arccosine (in radians) of a stream of `f32`.
///
/// Gives values in the range [0, pi] or not-a-number if outside range [-1, 1].
#[mel_treatment(
    input value Stream<f32>
    output acos Stream<f32>
)]
pub async fn acos() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| TryInto::<Vec<f32>>::try_into(values).unwrap())
    {
        check!(
            acos.send_many(
                values
                    .into_iter()
                    .map(|v| v.acos())
                    .collect::<VecDeque<_>>()
                    .into()
            )
            .await
        )
    }
}

/// Computes inverse hyperbolic cosine of a stream of `f32`.
#[mel_treatment(
    input value Stream<f32>
    output acosh Stream<f32>
)]
pub async fn acosh() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| TryInto::<Vec<f32>>::try_into(values).unwrap())
    {
        check!(
            acosh
                .send_many(
                    values
                        .into_iter()
                        .map(|v| v.acosh())
                        .collect::<VecDeque<_>>()
                        .into()
                )
                .await
        )
    }
}

/// Computes arcsine (in radians) of a stream of `f32`.
///
/// Gives values in the range [0, pi] or not-a-number if outside range [-1, 1].
#[mel_treatment(
    input value Stream<f32>
    output asin Stream<f32>
)]
pub async fn asin() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| TryInto::<Vec<f32>>::try_into(values).unwrap())
    {
        check!(
            asin.send_many(
                values
                    .into_iter()
                    .map(|v| v.asin())
                    .collect::<VecDeque<_>>()
                    .into()
            )
            .await
        )
    }
}

/// Computes inverse hyperbolic sine of a stream of `f32`.
#[mel_treatment(
    input value Stream<f32>
    output asinh Stream<f32>
)]
pub async fn asinh() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| TryInto::<Vec<f32>>::try_into(values).unwrap())
    {
        check!(
            asinh
                .send_many(
                    values
                        .into_iter()
                        .map(|v| v.asinh())
                        .collect::<VecDeque<_>>()
                        .into()
                )
                .await
        )
    }
}

/// Computes arctangent (in radians) of a stream of `f32`.
///
/// Gives values in the range [-pi/2, pi/2].
#[mel_treatment(
    input value Stream<f32>
    output atan Stream<f32>
)]
pub async fn atan() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| TryInto::<Vec<f32>>::try_into(values).unwrap())
    {
        check!(
            atan.send_many(
                values
                    .into_iter()
                    .map(|v| v.atan())
                    .collect::<VecDeque<_>>()
                    .into()
            )
            .await
        )
    }
}

/// Computes inverse hyperbolic tangent of a stream of  `f32`.
#[mel_treatment(
    input value Stream<f32>
    output atanh Stream<f32>
)]
pub async fn atanh() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| TryInto::<Vec<f32>>::try_into(values).unwrap())
    {
        check!(
            atanh
                .send_many(
                    values
                        .into_iter()
                        .map(|v| v.atanh())
                        .collect::<VecDeque<_>>()
                        .into()
                )
                .await
        )
    }
}

/// Computes cosine (in radians) of a stream of `f32`.
#[mel_treatment(
    input value Stream<f32>
    output cos Stream<f32>
)]
pub async fn cos() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| TryInto::<Vec<f32>>::try_into(values).unwrap())
    {
        check!(
            cos.send_many(
                values
                    .into_iter()
                    .map(|v| v.cos())
                    .collect::<VecDeque<_>>()
                    .into()
            )
            .await
        )
    }
}

/// Computes hyberbolic cosine of a stream of `f32`.
#[mel_treatment(
    input value Stream<f32>
    output cosh Stream<f32>
)]
pub async fn cosh() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| TryInto::<Vec<f32>>::try_into(values).unwrap())
    {
        check!(
            cosh.send_many(
                values
                    .into_iter()
                    .map(|v| v.cosh())
                    .collect::<VecDeque<_>>()
                    .into()
            )
            .await
        )
    }
}

/// Computes sine (in radians) of a stream of `f32`.
#[mel_treatment(
    input value Stream<f32>
    output sin Stream<f32>
)]
pub async fn sin() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| TryInto::<Vec<f32>>::try_into(values).unwrap())
    {
        check!(
            sin.send_many(
                values
                    .into_iter()
                    .map(|v| v.sin())
                    .collect::<VecDeque<_>>()
                    .into()
            )
            .await
        )
    }
}

/// Computes hyberbolic sine of a stream of `f32`.
#[mel_treatment(
    input value Stream<f32>
    output sinh Stream<f32>
)]
pub async fn sinh() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| TryInto::<Vec<f32>>::try_into(values).unwrap())
    {
        check!(
            sinh.send_many(
                values
                    .into_iter()
                    .map(|v| v.sinh())
                    .collect::<VecDeque<_>>()
                    .into()
            )
            .await
        )
    }
}

/// Computes tangent (in radians) of a stream of `f32`.
#[mel_treatment(
    input value Stream<f32>
    output tan Stream<f32>
)]
pub async fn tan() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| TryInto::<Vec<f32>>::try_into(values).unwrap())
    {
        check!(
            tan.send_many(
                values
                    .into_iter()
                    .map(|v| v.tan())
                    .collect::<VecDeque<_>>()
                    .into()
            )
            .await
        )
    }
}

/// Computes hyberbolic tangent of a stream of `f32`.
#[mel_treatment(
    input value Stream<f32>
    output tanh Stream<f32>
)]
pub async fn tanh() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| TryInto::<Vec<f32>>::try_into(values).unwrap())
    {
        check!(
            tanh.send_many(
                values
                    .into_iter()
                    .map(|v| v.tanh())
                    .collect::<VecDeque<_>>()
                    .into()
            )
            .await
        )
    }
}

/// Add `a` and `b`
#[mel_function]
pub fn add(a: f32, b: f32) -> f32 {
    a + b
}

/// Divide `dividend` by `divisor`
#[mel_function]
pub fn div(dividend: f32, divisor: f32) -> f32 {
    dividend / divisor
}

/// Multiply `a` by `b`
#[mel_function]
pub fn mult(a: f32, b: f32) -> f32 {
    a * b
}

/// Get the remainder of `dividend` by `divisor`
#[mel_function]
pub fn rem(dividend: f32, divisor: f32) -> f32 {
    dividend % divisor
}

/// Substract `b` from `a`
#[mel_function]
pub fn sub(a: f32, b: f32) -> f32 {
    a - b
}

/// Tells whether `a` is strictly greater than `b`.
#[mel_function]
pub fn gt(a: f32, b: f32) -> bool {
    a > b
}

/// Tells whether `a` is strictly lower than `b`.
#[mel_function]
pub fn lt(a: f32, b: f32) -> bool {
    a < b
}

/// Compares and gives the minimum of two values.
#[mel_function]
pub fn min(a: f32, b: f32) -> f32 {
    a.min(b)
}

/// Compares and gives the maximum of two values.
#[mel_function]
pub fn max(a: f32, b: f32) -> f32 {
    a.max(b)
}

/// Add values from two streams of `f32`.
///
/// Values passed through `a` & `b` are added and send in sum.
#[mel_treatment(
    input a Stream<f32>
    input b Stream<f32>
    output sum Stream<f32>
)]
pub async fn add() {
    while let (Ok(a), Ok(b)) = (
        a.recv_one()
            .await
            .map(|val| GetData::<f32>::try_data(val).unwrap()),
        b.recv_one()
            .await
            .map(|val| GetData::<f32>::try_data(val).unwrap()),
    ) {
        check!(sum.send_one((a + b).into()).await)
    }
}

/// Divide values from two streams of `f32`.
///
/// Every `a` number passed through the stream is divided by `b` counterpart.
#[mel_treatment(
    input a Stream<f32>
    input b Stream<f32>
    output quotient Stream<f32>
)]
pub async fn div() {
    while let (Ok(a), Ok(b)) = (
        a.recv_one()
            .await
            .map(|val| GetData::<f32>::try_data(val).unwrap()),
        b.recv_one()
            .await
            .map(|val| GetData::<f32>::try_data(val).unwrap()),
    ) {
        check!(quotient.send_one((a / b).into()).await)
    }
}

/// Multiply values from two streams of `f32`.
///
/// Every `a` number passed through the stream is multiplied by `b` counterpart.
#[mel_treatment(
    input a Stream<f32>
    input b Stream<f32>
    output product Stream<f32>
)]
pub async fn mult() {
    while let (Ok(a), Ok(b)) = (
        a.recv_one()
            .await
            .map(|val| GetData::<f32>::try_data(val).unwrap()),
        b.recv_one()
            .await
            .map(|val| GetData::<f32>::try_data(val).unwrap()),
    ) {
        check!(product.send_one((a * b).into()).await)
    }
}

/// Give the remainder of the division from two streams of `f32`.
///
/// Every `a` number passed through the stream is divided by `b` and the remainder is outputted.
#[mel_treatment(
    input a Stream<f32>
    input b Stream<f32>
    output remainder Stream<f32>
)]
pub async fn rem() {
    while let (Ok(a), Ok(b)) = (
        a.recv_one()
            .await
            .map(|val| GetData::<f32>::try_data(val).unwrap()),
        b.recv_one()
            .await
            .map(|val| GetData::<f32>::try_data(val).unwrap()),
    ) {
        check!(remainder.send_one((a % b).into()).await)
    }
}

/// Substract values from two streams of `f32`.
///
/// Every `a` number passed through the stream get `b` substracted.
#[mel_treatment(
    input a Stream<f32>
    input b Stream<f32>
    output diff Stream<f32>
)]
pub async fn sub() {
    while let (Ok(a), Ok(b)) = (
        a.recv_one()
            .await
            .map(|val| GetData::<f32>::try_data(val).unwrap()),
        b.recv_one()
            .await
            .map(|val| GetData::<f32>::try_data(val).unwrap()),
    ) {
        check!(diff.send_one((a - b).into()).await)
    }
}

/// Compares and gives the minimum of two values.
#[mel_treatment(
    input a Stream<f32>
    input b Stream<f32>
    output min Stream<f32>
)]
pub async fn min() {
    while let (Ok(a), Ok(b)) = (
        a.recv_one()
            .await
            .map(|val| GetData::<f32>::try_data(val).unwrap()),
        b.recv_one()
            .await
            .map(|val| GetData::<f32>::try_data(val).unwrap()),
    ) {
        check!(min.send_one(a.min(b).into()).await)
    }
}

/// Compares and gives the maximum of two values.
#[mel_treatment(
    input a Stream<f32>
    input b Stream<f32>
    output max Stream<f32>
)]
pub async fn max() {
    while let (Ok(a), Ok(b)) = (
        a.recv_one()
            .await
            .map(|val| GetData::<f32>::try_data(val).unwrap()),
        b.recv_one()
            .await
            .map(|val| GetData::<f32>::try_data(val).unwrap()),
    ) {
        check!(max.send_one(a.max(b).into()).await)
    }
}

/// Determine whether `a` is strictly lower than `b`
#[mel_treatment(
    input a Stream<f32>
    input b Stream<f32>
    output is Stream<bool>
)]
pub async fn lower_than() {
    while let (Ok(a), Ok(b)) = (
        a.recv_one()
            .await
            .map(|val| GetData::<f32>::try_data(val).unwrap()),
        b.recv_one()
            .await
            .map(|val| GetData::<f32>::try_data(val).unwrap()),
    ) {
        check!(is.send_one((a < b).into()).await)
    }
}

/// Determine whether `a` is strictly greater than `b`
#[mel_treatment(
    input a Stream<f32>
    input b Stream<f32>
    output is Stream<bool>
)]
pub async fn greater_than() {
    while let (Ok(a), Ok(b)) = (
        a.recv_one()
            .await
            .map(|val| GetData::<f32>::try_data(val).unwrap()),
        b.recv_one()
            .await
            .map(|val| GetData::<f32>::try_data(val).unwrap()),
    ) {
        check!(is.send_one((a > b).into()).await)
    }
}

/// Get absolute value
#[mel_function]
pub fn abs(value: f32) -> f32 {
    value.abs()
}

/// Get the absolute values from a stream of `f32`.
#[mel_treatment(
    input value Stream<f32>
    output abs Stream<f32>
)]
pub async fn abs() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| TryInto::<Vec<f32>>::try_into(values).unwrap())
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
