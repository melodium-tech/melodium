use melodium_core::*;
use melodium_macro::{check, mel_function, mel_treatment};

/// Tells if a value is not a number.
///
/// Returns `true` if value is `NaN`.
#[mel_function(
    generic F (Float)
)]
pub fn is_nan(value: F) -> bool {
    value.float_is_nan()
}

/// Tells if a stream contains not-a-number values.
///
/// Output `true` if value is `NaN`.  
#[mel_treatment(
    generic F (Float)
    input value Stream<F>
    output nan Stream<bool>
)]
pub async fn is_nan() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            nan.send_many(TransmissionValue::Bool(
                values.into_iter().map(|val| val.float_is_nan()).collect()
            ))
            .await
        )
    }
}

/// Tells if a value is finite.
///
/// Returns `true` if value is not infinite nor `NaN`.
#[mel_function(
    generic F (Float)
)]
pub fn is_finite(value: F) -> bool {
    value.float_is_finite()
}

/// Tells if a stream contains finite values.
///
/// Output `true` if value is not infinite nor `NaN`.  
#[mel_treatment(
    generic F (Float)
    input value Stream<F>
    output finite Stream<bool>
)]
pub async fn is_finite() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            finite
                .send_many(TransmissionValue::Bool(
                    values
                        .into_iter()
                        .map(|val| val.float_is_finite())
                        .collect()
                ))
                .await
        )
    }
}

/// Tells if a value is infinite.
#[mel_function(
    generic F (Float)
)]
pub fn is_infinite(value: F) -> bool {
    value.float_is_infinite()
}

/// Tells if a stream contains infinite values.
#[mel_treatment(
    generic F (Float)
    input value Stream<F>
    output infinite Stream<bool>
)]
pub async fn is_infinite() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            infinite
                .send_many(TransmissionValue::Bool(
                    values
                        .into_iter()
                        .map(|val| val.float_is_infinite())
                        .collect()
                ))
                .await
        )
    }
}

/// Tells if a value is normal.
///
/// Returns `true` if the number is neither zero, infinite, [subnormal](https://en.wikipedia.org/wiki/Subnormal_number), or NaN.
#[mel_function(
    generic F (Float)
)]
pub fn is_normal(value: F) -> bool {
    value.float_is_normal()
}

/// Tells if a stream contains normal values.
///
/// Output `true` if value is neither zero, infinite, [subnormal](https://en.wikipedia.org/wiki/Subnormal_number), or NaN.
#[mel_treatment(
    generic F (Float)
    input value Stream<F>
    output normal Stream<bool>
)]
pub async fn is_normal() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            normal
                .send_many(TransmissionValue::Bool(
                    values
                        .into_iter()
                        .map(|val| val.float_is_normal())
                        .collect()
                ))
                .await
        )
    }
}

/// Tells if a value is subnormal.
///
/// Returns `true` if the number is [subnormal](https://en.wikipedia.org/wiki/Subnormal_number).
#[mel_function(
    generic F (Float)
)]
pub fn is_subnormal(value: F) -> bool {
    value.float_is_subnormal()
}

/// Tells if a stream contains subnormal values.
///
/// Outputs `true` if the number is [subnormal](https://en.wikipedia.org/wiki/Subnormal_number).
#[mel_treatment(
    generic F (Float)
    input value Stream<F>
    output subnormal Stream<bool>
)]
pub async fn is_subnormal() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            subnormal
                .send_many(TransmissionValue::Bool(
                    values
                        .into_iter()
                        .map(|val| val.float_is_subnormal())
                        .collect()
                ))
                .await
        )
    }
}

/// Return largest integer less than or equal to `val`
#[mel_function(
    generic F (Float)
)]
pub fn floor(val: F) -> F {
    val.float_floor()
}

/// Computes largest integer less than or equal to of streamed values.
#[mel_treatment(
    generic F (Float)
    input value Stream<F>
    output floor Stream<F>
)]
pub async fn floor() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            floor
                .send_many(TransmissionValue::Other(
                    values.into_iter().map(|val| val.float_floor()).collect()
                ))
                .await
        )
    }
}
/// Return smallest integer greater than or equal to `val`
#[mel_function(
    generic F (Float)
)]
pub fn ceil(val: F) -> F {
    val.float_ceil()
}

/// Computes smallest integer greater than or equal to of streamed values.
#[mel_treatment(
    generic F (Float)
    input value Stream<F>
    output ceil Stream<F>
)]
pub async fn ceil() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            ceil.send_many(TransmissionValue::Other(
                values.into_iter().map(|val| val.float_ceil()).collect()
            ))
            .await
        )
    }
}
/// Return nearest integer to `val`
#[mel_function(
    generic F (Float)
)]
pub fn round(val: F) -> F {
    val.float_round()
}

/// Computes nearest integer to of streamed values.
#[mel_treatment(
    generic F (Float)
    input value Stream<F>
    output round Stream<F>
)]
pub async fn round() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            round
                .send_many(TransmissionValue::Other(
                    values.into_iter().map(|val| val.float_round()).collect()
                ))
                .await
        )
    }
}
/// Return integer part of `val`
#[mel_function(
    generic F (Float)
)]
pub fn trunc(val: F) -> F {
    val.float_trunc()
}

/// Computes integer part of of streamed values.
#[mel_treatment(
    generic F (Float)
    input value Stream<F>
    output trunc Stream<F>
)]
pub async fn trunc() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            trunc
                .send_many(TransmissionValue::Other(
                    values.into_iter().map(|val| val.float_trunc()).collect()
                ))
                .await
        )
    }
}
/// Return fractional part of `val`
#[mel_function(
    generic F (Float)
)]
pub fn fract(val: F) -> F {
    val.float_fract()
}

/// Computes fractional part of of streamed values.
#[mel_treatment(
    generic F (Float)
    input value Stream<F>
    output fract Stream<F>
)]
pub async fn fract() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            fract
                .send_many(TransmissionValue::Other(
                    values.into_iter().map(|val| val.float_fract()).collect()
                ))
                .await
        )
    }
}
/// Return inverse of `val`
#[mel_function(
    generic F (Float)
)]
pub fn recip(val: F) -> F {
    val.float_recip()
}

/// Computes inverse of of streamed values.
#[mel_treatment(
    generic F (Float)
    input value Stream<F>
    output recip Stream<F>
)]
pub async fn recip() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            recip
                .send_many(TransmissionValue::Other(
                    values.into_iter().map(|val| val.float_recip()).collect()
                ))
                .await
        )
    }
}

/// Compute exponent of given value
#[mel_function(
    generic F (Float)
)]
pub fn pow(val: F, exp: F) -> F {
    val.float_pow(&exp)
}

/// Compute exponent of streamed values
#[mel_treatment(
    generic F (Float)
    input value Stream<F>
    output pow Stream<F>
)]
pub async fn pow(exp: F) {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            pow.send_many(TransmissionValue::Other(
                values.into_iter().map(|val| val.float_pow(&exp)).collect()
            ))
            .await
        )
    }
}

/// Return square root of `val`
#[mel_function(
    generic F (Float)
)]
pub fn sqrt(val: F) -> F {
    val.float_sqrt()
}

/// Computes square root of of streamed values.
#[mel_treatment(
    generic F (Float)
    input value Stream<F>
    output sqrt Stream<F>
)]
pub async fn sqrt() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            sqrt.send_many(TransmissionValue::Other(
                values.into_iter().map(|val| val.float_sqrt()).collect()
            ))
            .await
        )
    }
}
/// Return exponential of `val`
#[mel_function(
    generic F (Float)
)]
pub fn exp(val: F) -> F {
    val.float_exp()
}

/// Computes exponential of of streamed values.
#[mel_treatment(
    generic F (Float)
    input value Stream<F>
    output exp Stream<F>
)]
pub async fn exp() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            exp.send_many(TransmissionValue::Other(
                values.into_iter().map(|val| val.float_exp()).collect()
            ))
            .await
        )
    }
}
/// Return 2 powered by  `val`
#[mel_function(
    generic F (Float)
)]
pub fn exp2(val: F) -> F {
    val.float_exp2()
}

/// Computes 2 powered by  of streamed values.
#[mel_treatment(
    generic F (Float)
    input value Stream<F>
    output exp2 Stream<F>
)]
pub async fn exp2() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            exp2.send_many(TransmissionValue::Other(
                values.into_iter().map(|val| val.float_exp2()).collect()
            ))
            .await
        )
    }
}
/// Return natural logarithm of `val`
#[mel_function(
    generic F (Float)
)]
pub fn ln(val: F) -> F {
    val.float_ln()
}

/// Computes natural logarithm of of streamed values.
#[mel_treatment(
    generic F (Float)
    input value Stream<F>
    output ln Stream<F>
)]
pub async fn ln() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            ln.send_many(TransmissionValue::Other(
                values.into_iter().map(|val| val.float_ln()).collect()
            ))
            .await
        )
    }
}

/// Compute logarithm of given value
#[mel_function(
    generic F (Float)
)]
pub fn log(val: F, base: F) -> F {
    val.float_log(&base)
}

/// Compute exponent of streamed values
#[mel_treatment(
    generic F (Float)
    input value Stream<F>
    output log Stream<F>
)]
pub async fn log(base: F) {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            log.send_many(TransmissionValue::Other(
                values.into_iter().map(|val| val.float_log(&base)).collect()
            ))
            .await
        )
    }
}

/// Return base 2 logarithm of `val`
#[mel_function(
    generic F (Float)
)]
pub fn log2(val: F) -> F {
    val.float_log2()
}

/// Computes base 2 logarithm of of streamed values.
#[mel_treatment(
    generic F (Float)
    input value Stream<F>
    output log2 Stream<F>
)]
pub async fn log2() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            log2.send_many(TransmissionValue::Other(
                values.into_iter().map(|val| val.float_log2()).collect()
            ))
            .await
        )
    }
}
/// Return base 10 logarithm of `val`
#[mel_function(
    generic F (Float)
)]
pub fn log10(val: F) -> F {
    val.float_log10()
}

/// Computes base 10 logarithm of of streamed values.
#[mel_treatment(
    generic F (Float)
    input value Stream<F>
    output log10 Stream<F>
)]
pub async fn log10() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            log10
                .send_many(TransmissionValue::Other(
                    values.into_iter().map(|val| val.float_log10()).collect()
                ))
                .await
        )
    }
}
/// Return cube root of `val`
#[mel_function(
    generic F (Float)
)]
pub fn cbrt(val: F) -> F {
    val.float_cbrt()
}

/// Computes cube root of of streamed values.
#[mel_treatment(
    generic F (Float)
    input value Stream<F>
    output cbrt Stream<F>
)]
pub async fn cbrt() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            cbrt.send_many(TransmissionValue::Other(
                values.into_iter().map(|val| val.float_cbrt()).collect()
            ))
            .await
        )
    }
}

/// Compute hypotenuse of given value
#[mel_function(
    generic F (Float)
)]
pub fn hypot(val: F, n: F) -> F {
    val.float_hypot(&n)
}

/// Compute exponent of streamed values
#[mel_treatment(
    generic F (Float)
    input value Stream<F>
    output hypot Stream<F>
)]
pub async fn hypot(n: F) {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            hypot
                .send_many(TransmissionValue::Other(
                    values.into_iter().map(|val| val.float_hypot(&n)).collect()
                ))
                .await
        )
    }
}

/// Return sine of `val`
#[mel_function(
    generic F (Float)
)]
pub fn sin(val: F) -> F {
    val.float_sin()
}

/// Computes sine of of streamed values.
#[mel_treatment(
    generic F (Float)
    input value Stream<F>
    output sin Stream<F>
)]
pub async fn sin() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            sin.send_many(TransmissionValue::Other(
                values.into_iter().map(|val| val.float_sin()).collect()
            ))
            .await
        )
    }
}
/// Return cosine of `val`
#[mel_function(
    generic F (Float)
)]
pub fn cos(val: F) -> F {
    val.float_cos()
}

/// Computes cosine of of streamed values.
#[mel_treatment(
    generic F (Float)
    input value Stream<F>
    output cos Stream<F>
)]
pub async fn cos() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            cos.send_many(TransmissionValue::Other(
                values.into_iter().map(|val| val.float_cos()).collect()
            ))
            .await
        )
    }
}
/// Return tangent of `val`
#[mel_function(
    generic F (Float)
)]
pub fn tan(val: F) -> F {
    val.float_tan()
}

/// Computes tangent of of streamed values.
#[mel_treatment(
    generic F (Float)
    input value Stream<F>
    output tan Stream<F>
)]
pub async fn tan() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            tan.send_many(TransmissionValue::Other(
                values.into_iter().map(|val| val.float_tan()).collect()
            ))
            .await
        )
    }
}
/// Return arcsine of `val`
#[mel_function(
    generic F (Float)
)]
pub fn asin(val: F) -> F {
    val.float_asin()
}

/// Computes arcsine of of streamed values.
#[mel_treatment(
    generic F (Float)
    input value Stream<F>
    output asin Stream<F>
)]
pub async fn asin() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            asin.send_many(TransmissionValue::Other(
                values.into_iter().map(|val| val.float_asin()).collect()
            ))
            .await
        )
    }
}
/// Return arcosine of `val`
#[mel_function(
    generic F (Float)
)]
pub fn acos(val: F) -> F {
    val.float_acos()
}

/// Computes arcosine of of streamed values.
#[mel_treatment(
    generic F (Float)
    input value Stream<F>
    output acos Stream<F>
)]
pub async fn acos() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            acos.send_many(TransmissionValue::Other(
                values.into_iter().map(|val| val.float_acos()).collect()
            ))
            .await
        )
    }
}
/// Return arctangent of `val`
#[mel_function(
    generic F (Float)
)]
pub fn atan(val: F) -> F {
    val.float_atan()
}

/// Computes arctangent of of streamed values.
#[mel_treatment(
    generic F (Float)
    input value Stream<F>
    output atan Stream<F>
)]
pub async fn atan() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            atan.send_many(TransmissionValue::Other(
                values.into_iter().map(|val| val.float_atan()).collect()
            ))
            .await
        )
    }
}

/// Compute the four quadrant arctangent of value and n
#[mel_function(
    generic F (Float)
)]
pub fn atan2(val: F, n: F) -> F {
    val.float_atan2(&n)
}

/// Compute the four quadrant arctangent of streamed values
#[mel_treatment(
    generic F (Float)
    input value Stream<F>
    output atan2 Stream<F>
)]
pub async fn atan2(n: F) {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            atan2
                .send_many(TransmissionValue::Other(
                    values.into_iter().map(|val| val.float_atan2(&n)).collect()
                ))
                .await
        )
    }
}

/// Return hyperbolic sine `val`
#[mel_function(
    generic F (Float)
)]
pub fn sinh(val: F) -> F {
    val.float_sinh()
}

/// Computes hyperbolic sine of streamed values.
#[mel_treatment(
    generic F (Float)
    input value Stream<F>
    output sinh Stream<F>
)]
pub async fn sinh() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            sinh.send_many(TransmissionValue::Other(
                values.into_iter().map(|val| val.float_sinh()).collect()
            ))
            .await
        )
    }
}
/// Return hyperbolic cosine `val`
#[mel_function(
    generic F (Float)
)]
pub fn cosh(val: F) -> F {
    val.float_cosh()
}

/// Computes hyperbolic cosine of streamed values.
#[mel_treatment(
    generic F (Float)
    input value Stream<F>
    output cosh Stream<F>
)]
pub async fn cosh() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            cosh.send_many(TransmissionValue::Other(
                values.into_iter().map(|val| val.float_cosh()).collect()
            ))
            .await
        )
    }
}
/// Return hyperbolic tangent `val`
#[mel_function(
    generic F (Float)
)]
pub fn tanh(val: F) -> F {
    val.float_tanh()
}

/// Computes hyperbolic tangent of streamed values.
#[mel_treatment(
    generic F (Float)
    input value Stream<F>
    output tanh Stream<F>
)]
pub async fn tanh() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            tanh.send_many(TransmissionValue::Other(
                values.into_iter().map(|val| val.float_tanh()).collect()
            ))
            .await
        )
    }
}
/// Return inverse hyperbolic sine `val`
#[mel_function(
    generic F (Float)
)]
pub fn asinh(val: F) -> F {
    val.float_asinh()
}

/// Computes inverse hyperbolic sine of streamed values.
#[mel_treatment(
    generic F (Float)
    input value Stream<F>
    output asinh Stream<F>
)]
pub async fn asinh() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            asinh
                .send_many(TransmissionValue::Other(
                    values.into_iter().map(|val| val.float_asinh()).collect()
                ))
                .await
        )
    }
}
/// Return inverse hyperbolic cosine `val`
#[mel_function(
    generic F (Float)
)]
pub fn acosh(val: F) -> F {
    val.float_acosh()
}

/// Computes inverse hyperbolic cosine of streamed values.
#[mel_treatment(
    generic F (Float)
    input value Stream<F>
    output acosh Stream<F>
)]
pub async fn acosh() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            acosh
                .send_many(TransmissionValue::Other(
                    values.into_iter().map(|val| val.float_acosh()).collect()
                ))
                .await
        )
    }
}
/// Return inverse hyperbolic tangent `val`
#[mel_function(
    generic F (Float)
)]
pub fn atanh(val: F) -> F {
    val.float_atanh()
}

/// Computes inverse hyperbolic tangent of streamed values.
#[mel_treatment(
    generic F (Float)
    input value Stream<F>
    output atanh Stream<F>
)]
pub async fn atanh() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            atanh
                .send_many(TransmissionValue::Other(
                    values.into_iter().map(|val| val.float_atanh()).collect()
                ))
                .await
        )
    }
}
/// Return conversion to degrees of `val`
#[mel_function(
    generic F (Float)
)]
pub fn to_degrees(val: F) -> F {
    val.float_to_degrees()
}

/// Computes conversion to degrees of streamed values.
#[mel_treatment(
    generic F (Float)
    input value Stream<F>
    output to_degrees Stream<F>
)]
pub async fn to_degrees() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            to_degrees
                .send_many(TransmissionValue::Other(
                    values
                        .into_iter()
                        .map(|val| val.float_to_degrees())
                        .collect()
                ))
                .await
        )
    }
}
/// Return conversion to radians of `val`
#[mel_function(
    generic F (Float)
)]
pub fn to_radians(val: F) -> F {
    val.float_to_radians()
}

/// Computes conversion to radians of streamed values.
#[mel_treatment(
    generic F (Float)
    input value Stream<F>
    output to_radians Stream<F>
)]
pub async fn to_radians() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            to_radians
                .send_many(TransmissionValue::Other(
                    values
                        .into_iter()
                        .map(|val| val.float_to_radians())
                        .collect()
                ))
                .await
        )
    }
}
