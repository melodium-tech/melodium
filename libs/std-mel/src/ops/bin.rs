use melodium_core::*;
use melodium_macro::{check, mel_function, mel_treatment};

/// Logical _and_
///
/// Makes _and_ ⋀ binary operation
#[mel_function(
    generic B (Binary)
)]
pub fn and(a: B, b: B) -> B {
    a.binary_and(&b)
}

/// Logical _and_
///
/// Makes _and_ ⋀ binary operation between values passed through streams.
#[mel_treatment(
    generic B (Binary)
    input a Stream<B>
    input b Stream<B>
    output and Stream<B>
)]
pub async fn and() {
    while let (Ok(a), Ok(b)) = (a.recv_one().await, b.recv_one().await) {
        check!(and.send_one(a.binary_and(&b)).await)
    }
}

/// Logical inclusive _or_
///
/// Makes _or_ ⋁ binary operation
#[mel_function(
    generic B (Binary)
)]
pub fn or(a: B, b: B) -> B {
    a.binary_or(&b)
}

/// Logical inclusive _or_.
///
/// Makes _or_ ⋁ binary operation between values passed through streams.
#[mel_treatment(
    generic B (Binary)
    input a Stream<B>
    input b Stream<B>
    output or Stream<B>
)]
pub async fn or() {
    while let (Ok(a), Ok(b)) = (a.recv_one().await, b.recv_one().await) {
        check!(or.send_one(a.binary_or(&b)).await)
    }
}

/// Logical exclusive _or_
///
/// Makes _xor_ ⊕ binary operation
#[mel_function(
    generic B (Binary)
)]
pub fn xor(a: B, b: B) -> B {
    a.binary_xor(&b)
}

/// Logical exclusive _or_.
///
/// Makes _xor_ ⊕ binary operation between values passed through streams.
#[mel_treatment(
    generic B (Binary)
    input a Stream<B>
    input b Stream<B>
    output or Stream<B>
)]
pub async fn xor() {
    while let (Ok(a), Ok(b)) = (a.recv_one().await, b.recv_one().await) {
        check!(or.send_one(a.binary_xor(&b)).await)
    }
}

/// Inverse binary value.
///
/// Apply _not_ ¬ binary operation.
#[mel_function(
    generic B (Binary)
)]
pub fn not(value: B) -> B {
    value.binary_not()
}

/// Inverse binary values of a stream.
///
/// Apply _not_ ¬ binary operation on all values in the stream.
#[mel_treatment(
    generic B (Binary)
    input value Stream<B>
    output not Stream<B>
)]
pub async fn not() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            not.send_many(TransmissionValue::Other(
                values.into_iter().map(|val| val.binary_not()).collect()
            ))
            .await
        )
    }
}
