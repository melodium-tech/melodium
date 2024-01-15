use melodium_core::*;
use melodium_macro::{check, mel_function, mel_treatment};

/// Return absolute value.
///
/// If the absolute value cannot fit into the type itself (such as `i8` `-128` that cannot be turned into `128`), none value is returned.
#[mel_function(
    generic N (Signed)
)]
pub fn abs(value: N) -> Option<N> {
    value.signed_abs()
}

/// Give the absolute values of a stream.
///
/// If the absolute value cannot fit into the type itself (such as `i8` `-128` that cannot be turned into `128`), none value is returned.
#[mel_treatment(
    generic N (Signed)
    input value Stream<N>
    output abs Stream<Option<N>>
)]
pub async fn abs() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            abs.send_many(TransmissionValue::Other(
                values
                    .into_iter()
                    .map(|val| val.signed_abs().into())
                    .collect()
            ))
            .await
        )
    }
}

/// Return number representing sign of given value.
///
/// - `0` if number is 0,
/// - `-1` if number is negative,
/// - `1` if number is positive.
///
/// ℹ️ For floating types (`f32` and `f64`), `NaN` values gives `NaN` output, `+INFINITY` and `+0.0` gives `1`, `-INFINITY` and `-0.0` gives `-1`.
#[mel_function(
    generic N (Signed)
)]
pub fn signum(value: N) -> N {
    value.signed_signum()
}

/// Gives numeric sign of a stream.
///
/// - `0` if number is 0,
/// - `-1` if number is negative,
/// - `1` if number is positive.
///
/// ℹ️ For floating types (`f32` and `f64`), `NaN` values gives `NaN` output, `+INFINITY` and `+0.0` gives `1`, `-INFINITY` and `-0.0` gives `-1`.
#[mel_treatment(
    generic N (Signed)
    input value Stream<N>
    output sign Stream<N>
)]
pub async fn signum() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            sign.send_many(TransmissionValue::Other(
                values.into_iter().map(|val| val.signed_signum()).collect()
            ))
            .await
        )
    }
}

/// Tells if a value is positive.
///
/// Returns `true` for strictly positive integers, and `false` for `0` and negative ones.  
/// ℹ️ For floating types (`f32` and `f64`), `NaN` values gives `false`, `+INFINITY` and `+0.0` gives `true`.
#[mel_function(
    generic N (Signed)
)]
pub fn is_positive(value: N) -> bool {
    value.signed_is_positive()
}

/// Tells if a stream contains positive values.
///
/// Output `true` for strictly positive integers, and `false` for `0` and negative ones.  
/// ℹ️ For floating types (`f32` and `f64`), `NaN` values gives `false`, `+INFINITY` and `+0.0` gives `true`.
#[mel_treatment(
    generic N (Signed)
    input value Stream<N>
    output positive Stream<bool>
)]
pub async fn is_positive() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            positive
                .send_many(TransmissionValue::Bool(
                    values
                        .into_iter()
                        .map(|val| val.signed_is_positive().into())
                        .collect()
                ))
                .await
        )
    }
}

/// Tells if a value is negative.
///
/// Returns `true` for strictly negative integers, and `false` for `0` and positive ones.  
/// ℹ️ For floating types (`f32` and `f64`), `NaN` values gives `false`, `-INFINITY` and `-0.0` gives `true`.
#[mel_function(
    generic N (Signed)
)]
pub fn is_negative(value: N) -> bool {
    value.signed_is_negative()
}

/// Tells if a stream contains negative values.
///
/// Output `true` for strictly negative integers, and `false` for `0` and positive ones.  
/// ℹ️ For floating types (`f32` and `f64`), `NaN` values gives `false`, `-INFINITY` and `-0.0` gives `true`.
#[mel_treatment(
    generic N (Signed)
    input value Stream<N>
    output negative Stream<bool>
)]
pub async fn is_negative() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            negative
                .send_many(TransmissionValue::Bool(
                    values
                        .into_iter()
                        .map(|val| val.signed_is_negative().into())
                        .collect()
                ))
                .await
        )
    }
}

/// Add `a` and `b`
///
/// This function is infaillible but may overflow if `a + b` is out of bounds for the data type.
#[mel_function(
    generic N (Add)
)]
pub fn add(a: N, b: N) -> N {
    a.add(&b)
}

/// Add values from two streams.
///
/// Values passed through `a` & `b` are added and send in sum.
/// This treatment is infaillible but output may overflow if `a + b` is out of bounds for the data type.
#[mel_treatment(
    generic N (Add)
    input a Stream<N>
    input b Stream<N>
    output sum Stream<N>
)]
pub async fn add() {
    while let (Ok(a), Ok(b)) = (a.recv_one().await, b.recv_one().await) {
        check!(sum.send_one(a.add(&b)).await)
    }
}

/// Add `a` and `b`, checking if overflow occurs
///
/// This function returns an option containing `a + b`, or none if result cause overflow in data type.
#[mel_function(
    generic N (CheckedAdd)
)]
pub fn checked_add(a: N, b: N) -> Option<N> {
    a.checked_add(&b)
}

/// Add values from two streams, checking if overflow occurs.
///
/// Values passed through `a` & `b` are added and send in sum.
/// This treatment outputs an option containing `a + b`, or none if result cause overflow in data type.
#[mel_treatment(
    generic N (CheckedAdd)
    input a Stream<N>
    input b Stream<N>
    output sum Stream<Option<N>>
)]
pub async fn checked_add() {
    while let (Ok(a), Ok(b)) = (a.recv_one().await, b.recv_one().await) {
        check!(sum.send_one(a.checked_add(&b).into()).await)
    }
}

/// Add `a` and `b`, saturating to bounds.
///
/// This function is infaillible and saturate to the closest bound, minimal or maximal, if `a + b` is out of bounds for the data type.
#[mel_function(
    generic N (SaturatingAdd)
)]
pub fn saturating_add(a: N, b: N) -> N {
    a.saturating_add(&b)
}

/// Add values from two streams, saturating to bounds.
///
/// Values passed through `a` & `b` are added and send in sum.
/// This treatment is infaillible and saturate to the closest bound, minimal or maximal, if `a + b` is out of bounds for the data type.
#[mel_treatment(
    generic N (SaturatingAdd)
    input a Stream<N>
    input b Stream<N>
    output sum Stream<N>
)]
pub async fn saturating_add() {
    while let (Ok(a), Ok(b)) = (a.recv_one().await, b.recv_one().await) {
        check!(sum.send_one(a.saturating_add(&b)).await)
    }
}

/// Add `a` and `b`, wrapping on bounds.
///
/// This function is infaillible and wrap if `a + b` reach boundary of the data type.
#[mel_function(
    generic N (WrappingAdd)
)]
pub fn wrapping_add(a: N, b: N) -> N {
    a.wrapping_add(&b)
}

/// Add values from two streams, wrapping on bounds.
///
/// Values passed through `a` & `b` are added and send in sum.
/// This treatment is infaillible and wrap if `a + b` reach boundary of the data type.
#[mel_treatment(
    generic N (WrappingAdd)
    input a Stream<N>
    input b Stream<N>
    output sum Stream<N>
)]
pub async fn wrapping_add() {
    while let (Ok(a), Ok(b)) = (a.recv_one().await, b.recv_one().await) {
        check!(sum.send_one(a.wrapping_add(&b)).await)
    }
}

/// Sustract `b` from `a`
///
/// This function is infaillible but may overflow if `a - b` is out of bounds for the data type.
#[mel_function(
    generic N (Sub)
)]
pub fn sub(a: N, b: N) -> N {
    a.sub(&b)
}

/// Substract values from two streams.
///
/// Values passed through `b` are substracted to `a` and send in diff.
/// This treatment is infaillible but output may overflow if `a - b` is out of bounds for the data type.
#[mel_treatment(
    generic N (Add)
    input a Stream<N>
    input b Stream<N>
    output diff Stream<N>
)]
pub async fn sub() {
    while let (Ok(a), Ok(b)) = (a.recv_one().await, b.recv_one().await) {
        check!(diff.send_one(a.sub(&b)).await)
    }
}

/// Sustract `b` from `a`, checking if overflow occurs
///
/// This function returns an option containing `a - b`, or none if result cause overflow in data type.
#[mel_function(
    generic N (CheckedSub)
)]
pub fn checked_sub(a: N, b: N) -> Option<N> {
    a.checked_sub(&b)
}

/// Substract values from two streams, checking if overflow occurs.
///
/// Values passed through `b` are substracted to `a` and send in diff.
/// This treatment outputs an option containing `a - b`, or none if result cause overflow in data type.
#[mel_treatment(
    generic N (CheckedSub)
    input a Stream<N>
    input b Stream<N>
    output diff Stream<Option<N>>
)]
pub async fn checked_sub() {
    while let (Ok(a), Ok(b)) = (a.recv_one().await, b.recv_one().await) {
        check!(diff.send_one(a.checked_sub(&b).into()).await)
    }
}

/// Sustract `b` from `a`, saturating to bounds.
///
/// This function is infaillible and saturate to the closest bound, minimal or maximal, if `a - b` is out of bounds for the data type.
#[mel_function(
    generic N (SaturatingSub)
)]
pub fn saturating_sub(a: N, b: N) -> N {
    a.saturating_sub(&b)
}

/// Substract values from two streams, saturating to bounds.
///
/// Values passed through `b` are substracted to `a` and send in diff.
/// This treatment is infaillible and saturate to the closest bound, minimal or maximal, if `a - b` is out of bounds for the data type.
#[mel_treatment(
    generic N (SaturatingSub)
    input a Stream<N>
    input b Stream<N>
    output diff Stream<N>
)]
pub async fn saturating_sub() {
    while let (Ok(a), Ok(b)) = (a.recv_one().await, b.recv_one().await) {
        check!(diff.send_one(a.saturating_sub(&b)).await)
    }
}

/// Sustract `b` from `a`, wrapping on bounds.
///
/// This function is infaillible and wrap if `a - b` reach boundary of the data type.
#[mel_function(
    generic N (WrappingSub)
)]
pub fn wrapping_sub(a: N, b: N) -> N {
    a.wrapping_sub(&b)
}

/// Substract values from two streams, wrapping on bounds.
///
/// Values passed through `b` are substracted to `a` and send in diff.
/// This treatment is infaillible and wrap if `a - b` reach boundary of the data type.
#[mel_treatment(
    generic N (WrappingSub)
    input a Stream<N>
    input b Stream<N>
    output diff Stream<N>
)]
pub async fn wrapping_sub() {
    while let (Ok(a), Ok(b)) = (a.recv_one().await, b.recv_one().await) {
        check!(diff.send_one(a.wrapping_sub(&b)).await)
    }
}

/// Multiply `a` and `b`
///
/// This function is infaillible but may overflow if `a × b` is out of bounds for the data type.
#[mel_function(
    generic N (Mul)
)]
pub fn mul(a: N, b: N) -> N {
    a.mul(&b)
}

/// Multiply values from two streams.
///
/// Values passed through `a` & `b` are multiplied and send in prod.
/// This treatment is infaillible but output may overflow if `a × b` is out of bounds for the data type.
#[mel_treatment(
    generic N (Mul)
    input a Stream<N>
    input b Stream<N>
    output prod Stream<N>
)]
pub async fn mul() {
    while let (Ok(a), Ok(b)) = (a.recv_one().await, b.recv_one().await) {
        check!(prod.send_one(a.mul(&b)).await)
    }
}

/// Multiply `a` and `b`, checking if overflow occurs
///
/// This function returns an option containing `a × b`, or none if result cause overflow in data type.
#[mel_function(
    generic N (CheckedMul)
)]
pub fn checked_mul(a: N, b: N) -> Option<N> {
    a.checked_mul(&b)
}

/// Multiply values from two streams, checking if overflow occurs.
///
/// Values passed through `a` & `b` are multiplied and send in prod.
/// This treatment outputs an option containing `a × b`, or none if result cause overflow in data type.
#[mel_treatment(
    generic N (CheckedMul)
    input a Stream<N>
    input b Stream<N>
    output prod Stream<Option<N>>
)]
pub async fn checked_mul() {
    while let (Ok(a), Ok(b)) = (a.recv_one().await, b.recv_one().await) {
        check!(prod.send_one(a.checked_mul(&b).into()).await)
    }
}

/// Multiply `a` and `b`, saturating to bounds.
///
/// This function is infaillible and saturate to the closest bound, minimal or maximal, if `a × b` is out of bounds for the data type.
#[mel_function(
    generic N (SaturatingMul)
)]
pub fn saturating_mul(a: N, b: N) -> N {
    a.saturating_mul(&b)
}

/// Multiply values from two streams, saturating to bounds.
///
/// Values passed through `a` & `b` are multiplied and send in prod.
/// This treatment is infaillible and saturate to the closest bound, minimal or maximal, if `a × b` is out of bounds for the data type.
#[mel_treatment(
    generic N (SaturatingMul)
    input a Stream<N>
    input b Stream<N>
    output prod Stream<N>
)]
pub async fn saturating_mul() {
    while let (Ok(a), Ok(b)) = (a.recv_one().await, b.recv_one().await) {
        check!(prod.send_one(a.saturating_mul(&b)).await)
    }
}

/// Multiply `a` and `b`, wrapping on bounds.
///
/// This function is infaillible and wrap if `a × b` reach boundary of the data type.
#[mel_function(
    generic N (WrappingMul)
)]
pub fn wrapping_mul(a: N, b: N) -> N {
    a.wrapping_mul(&b)
}

/// Multiply values from two streams, wrapping on bounds.
///
/// Values passed through `a` & `b` are multiplied and send in prod.
/// This treatment is infaillible and wrap if `a × b` reach boundary of the data type.
#[mel_treatment(
    generic N (WrappingMul)
    input a Stream<N>
    input b Stream<N>
    output prod Stream<N>
)]
pub async fn wrapping_mul() {
    while let (Ok(a), Ok(b)) = (a.recv_one().await, b.recv_one().await) {
        check!(prod.send_one(a.wrapping_mul(&b)).await)
    }
}

/// Divide `a` by `b`
///
/// This function is infaillible but may overflow if `a ÷ b` is out of bounds for the data type.  
/// ⚠️ For integers, this function returns `0` for divisions by `0`.  
/// ℹ️ For floating types, this function return infinity for divisions by `0`.
#[mel_function(
    generic N (Div)
)]
pub fn div(a: N, b: N) -> N {
    a.div(&b)
}

/// Divide values from two streams.
///
/// Values passed through `a` are divided by `b` and send in quot.
/// This treatment is infaillible but may overflow if `a ÷ b` is out of bounds for the data type.  
/// ⚠️ For integers, this treatment outputs `0` for divisions by `0`.  
/// ℹ️ For floating types, this treatment return infinity for divisions by `0`.
#[mel_treatment(
    generic N (Div)
    input a Stream<N>
    input b Stream<N>
    output quot Stream<N>
)]
pub async fn div() {
    while let (Ok(a), Ok(b)) = (a.recv_one().await, b.recv_one().await) {
        check!(quot.send_one(a.div(&b)).await)
    }
}

/// Divide `a` by `b`, checking if division is possible or if overflow occurs
///
/// This function returns an option containing `a ÷ b`, or none if division is not possible (as division by 0), or result cause overflow in data type.
#[mel_function(
    generic N (CheckedDiv)
)]
pub fn checked_div(a: N, b: N) -> Option<N> {
    a.checked_div(&b)
}

/// Divide values from two streams, checking if division is possible or if overflow occurs.
///
/// Values passed through `a` are divided by `b` and send in quot.
/// This treatment outputs an option containing `a ÷ b`, or none if division is not possible (as division by 0), or result cause overflow in data type.
#[mel_treatment(
    generic N (CheckedDiv)
    input a Stream<N>
    input b Stream<N>
    output quot Stream<Option<N>>
)]
pub async fn checked_div() {
    while let (Ok(a), Ok(b)) = (a.recv_one().await, b.recv_one().await) {
        check!(quot.send_one(a.checked_div(&b).into()).await)
    }
}

/// Gives remainder of `a` divided by `b`
///
/// This function is infaillible but may overflow if `a _mod_ b` is out of bounds for the data type.  
/// ⚠️ For integers, this function returns `0` for divisions by `0`.  
/// ℹ️ For floating types, this function return not-a-number (`NaN`) for divisions by `0`.
#[mel_function(
    generic N (Rem)
)]
pub fn rem(a: N, b: N) -> N {
    a.rem(&b)
}

/// Gives remainder of division from two streams.
///
/// Remainder is computed for values passed through `a` divided by `b` and send in rem.
/// This treatment is infaillible but may overflow if `a _mod_ b` is out of bounds for the data type.  
/// ⚠️ For integers, this treatment outputs `0` for divisions by `0`.  
/// ℹ️ For floating types, this treatment return not-a-number (`NaN`) for divisions by `0`.
#[mel_treatment(
    generic N (Rem)
    input a Stream<N>
    input b Stream<N>
    output rem Stream<N>
)]
pub async fn rem() {
    while let (Ok(a), Ok(b)) = (a.recv_one().await, b.recv_one().await) {
        check!(rem.send_one(a.rem(&b)).await)
    }
}

/// Gives remainder of `a` divided by `b`, checking if division is possible or if overflow occurs
///
/// This function returns an option containing `a _mod_ b`, or none if division is not possible (as division by 0), or result cause overflow in data type.
#[mel_function(
    generic N (CheckedRem)
)]
pub fn checked_rem(a: N, b: N) -> Option<N> {
    a.checked_rem(&b)
}

/// Gives remainder of division from two streams, checking if division is possible or if overflow occurs.
///
/// Remainder is computed for values passed through `a` divided by `b` and send in rem.
/// This treatment outputs an option containing `a _mod_ b`, or none if division is not possible (as division by 0), or result cause overflow in data type.
#[mel_treatment(
    generic N (CheckedRem)
    input a Stream<N>
    input b Stream<N>
    output rem Stream<Option<N>>
)]
pub async fn checked_rem() {
    while let (Ok(a), Ok(b)) = (a.recv_one().await, b.recv_one().await) {
        check!(rem.send_one(a.checked_rem(&b).into()).await)
    }
}

/// Return negation of given value
///
/// This function is infaillible but may overflow if `-val` is out of bounds for the data type.
#[mel_function(
    generic N (Neg)
)]
pub fn neg(val: N) -> N {
    val.neg()
}

/// Give negation of streamed values
///
/// This treatment is infaillible but output may overflow if `-value` is out of bounds for the data type.
#[mel_treatment(
    generic N (Neg)
    input value Stream<N>
    output neg Stream<N>
)]
pub async fn neg() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            neg.send_many(TransmissionValue::Other(
                values.into_iter().map(|val| val.neg()).collect()
            ))
            .await
        )
    }
}

/// Return negation of given value, checking if overflow occurs
///
/// This function returns an option containing `-val`, or none if result cause overflow in data type.
#[mel_function(
    generic N (CheckedNeg)
)]
pub fn checked_neg(val: N) -> Option<N> {
    val.checked_neg()
}

/// Give negation of streamed values, checking if overflow occurs.
///
/// This treatment outputs an option containing `-value`, or none if result cause overflow in data type.
#[mel_treatment(
    generic N (CheckedNeg)
    input value Stream<N>
    output neg Stream<Option<N>>
)]
pub async fn checked_neg() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            neg.send_many(TransmissionValue::Other(
                values
                    .into_iter()
                    .map(|val| val.checked_neg().into())
                    .collect()
            ))
            .await
        )
    }
}

/// Return negation of given value, wrapping on bounds.
///
/// This function is infaillible and wrap if `-val` reach boundary of the data type.
#[mel_function(
    generic N (WrappingNeg)
)]
pub fn wrapping_neg(val: N) -> N {
    val.wrapping_neg()
}

/// Give negation of streamed values, wrapping on bounds.
///
/// This treatment is infaillible and wrap if `-value` reach boundary of the data type.
#[mel_treatment(
    generic N (WrappingNeg)
    input value Stream<N>
    output neg Stream<N>
)]
pub async fn wrapping_neg() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            neg.send_many(TransmissionValue::Other(
                values
                    .into_iter()
                    .map(|val| val.wrapping_neg().into())
                    .collect()
            ))
            .await
        )
    }
}

/// Compute exponent of given value
///
/// This function is infaillible but may overflow if `valᵉˣᵖ` is out of bounds for the data type.
#[mel_function(
    generic N (Pow)
)]
pub fn pow(val: N, exp: u32) -> N {
    val.pow(&exp)
}

/// Compute exponent of streamed values
///
/// This treatment is infaillible but output may overflow if `valueᵉˣᵖ` is out of bounds for the data type.
#[mel_treatment(
    generic N (Pow)
    input value Stream<N>
    output pow Stream<N>
)]
pub async fn pow(exp: u32) {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            pow.send_many(TransmissionValue::Other(
                values.into_iter().map(|val| val.pow(&exp)).collect()
            ))
            .await
        )
    }
}

/// Compute exponent of given value, checking if overflow occurs
///
/// This function returns an option containing `valᵉˣᵖ`, or none if result cause overflow in data type.
#[mel_function(
    generic N (CheckedPow)
)]
pub fn checked_pow(val: N, exp: u32) -> Option<N> {
    val.checked_pow(&exp)
}

/// Compute exponent of streamed values, checking if overflow occurs.
///
/// This treatment outputs an option containing `valᵉˣᵖ`, or none if result cause overflow in data type.
#[mel_treatment(
    generic N (CheckedPow)
    input value Stream<N>
    output pow Stream<Option<N>>
)]
pub async fn checked_pow(exp: u32) {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            pow.send_many(TransmissionValue::Other(
                values
                    .into_iter()
                    .map(|val| val.checked_pow(&exp).into())
                    .collect()
            ))
            .await
        )
    }
}

/// Proceed to euclidian division of `a` by `b`
///
/// This function is infaillible but may overflow if `a ÷ b` is out of bounds for the data type.  
/// ⚠️ For integers, this function returns `0` for divisions by `0`.  
/// ℹ️ For floating types, this function return infinity for divisions by `0`.
#[mel_function(
    generic N (Euclid)
)]
pub fn euclid_div(a: N, b: N) -> N {
    a.euclid_div(&b)
}

/// Proceed to euclidian division from two streams.
///
/// Values passed through `a` are divided by `b` and send in quot.
/// This treatment is infaillible but may overflow if `a ÷ b` is out of bounds for the data type.  
/// ⚠️ For integers, this treatment outputs `0` for divisions by `0`.  
/// ℹ️ For floating types, this treatment return infinity for divisions by `0`.
#[mel_treatment(
    generic N (Euclid)
    input a Stream<N>
    input b Stream<N>
    output quot Stream<N>
)]
pub async fn euclid_div() {
    while let (Ok(a), Ok(b)) = (a.recv_one().await, b.recv_one().await) {
        check!(quot.send_one(a.euclid_div(&b)).await)
    }
}

/// Gives euclidian remainder of `a` divided by `b`
///
/// This function is infaillible but may overflow if `a _mod_ b` is out of bounds for the data type.  
/// ⚠️ For integers, this function returns `0` for divisions by `0`.  
/// ℹ️ For floating types, this function return not-a-number (`NaN`) for divisions by `0`.
#[mel_function(
    generic N (Euclid)
)]
pub fn euclid_rem(a: N, b: N) -> N {
    a.euclid_rem(&b)
}

/// Gives euclidian remainder of division from two streams.
///
/// Remainder is computed for values passed through `a` divided by `b` and send in rem.
/// This treatment is infaillible but may overflow if `a _mod_ b` is out of bounds for the data type.  
/// ⚠️ For integers, this treatment outputs `0` for divisions by `0`.  
/// ℹ️ For floating types, this treatment return not-a-number (`NaN`) for divisions by `0`.
#[mel_treatment(
    generic N (Euclid)
    input a Stream<N>
    input b Stream<N>
    output rem Stream<N>
)]
pub async fn euclid_rem() {
    while let (Ok(a), Ok(b)) = (a.recv_one().await, b.recv_one().await) {
        check!(rem.send_one(a.euclid_rem(&b)).await)
    }
}

/// Proceed to euclidian division of `a` by `b`, checking if division is possible or if overflow occurs
///
/// This function returns an option containing `a ÷ b`, or none if division is not possible (as division by 0), or result cause overflow in data type.
/// ℹ️ For floating types, this function return infinity for divisions by `0`.
#[mel_function(
    generic N (CheckedEuclid)
)]
pub fn checked_euclid_div(a: N, b: N) -> Option<N> {
    a.checked_euclid_div(&b)
}

/// Proceed to euclidian division from two streams, checking if division is possible or if overflow occurs.
///
/// Values passed through `a` are divided by `b` and send in quot.
/// This treatment outputs an option containing `a ÷ b`, or none if division is not possible (as division by 0), or result cause overflow in data type.
/// ℹ️ For floating types, this function return infinity for divisions by `0`.
#[mel_treatment(
    generic N (CheckedEuclid)
    input a Stream<N>
    input b Stream<N>
    output quot Stream<Option<N>>
)]
pub async fn checked_euclid_div() {
    while let (Ok(a), Ok(b)) = (a.recv_one().await, b.recv_one().await) {
        check!(quot.send_one(a.checked_euclid_div(&b).into()).await)
    }
}

/// Gives euclidian remainder of `a` divided by `b`, checking if division is possible or if overflow occurs
///
/// This function returns an option containing `a _mod_ b`, or none if division is not possible (as division by 0), or result cause overflow in data type.
#[mel_function(
    generic N (CheckedEuclid)
)]
pub fn checked_euclid_rem(a: N, b: N) -> Option<N> {
    a.checked_euclid_rem(&b)
}

/// Gives euclidian remainder of division from two streams, checking if division is possible or if overflow occurs.
///
/// Remainder is computed for values passed through `a` divided by `b` and send in rem.
/// This treatment outputs an option containing `a _mod_ b`, or none if division is not possible (as division by 0), or result cause overflow in data type.
#[mel_treatment(
    generic N (CheckedEuclid)
    input a Stream<N>
    input b Stream<N>
    output rem Stream<Option<N>>
)]
pub async fn checked_euclid_rem() {
    while let (Ok(a), Ok(b)) = (a.recv_one().await, b.recv_one().await) {
        check!(rem.send_one(a.checked_euclid_rem(&b).into()).await)
    }
}
