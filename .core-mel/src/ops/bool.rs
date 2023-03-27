use melodium_core::*;
use melodium_macro::{check, mel_function, mel_treatment};

/// Return whether `a` is equal to `b`
#[mel_function]
pub fn equal(a: bool, b: bool) -> bool {
    a == b
}

/// Return whether `a` is different `b`
#[mel_function]
pub fn not_equal(a: bool, b: bool) -> bool {
    a != b
}

/// Determine whether `a` is equal to `b`
#[mel_treatment(
    input a Stream<bool>
    input b Stream<bool>
    output result Stream<bool>
)]
pub async fn equal() {
    while let (Ok(a), Ok(b)) = (a.recv_one_bool().await, b.recv_one_bool().await) {
        check!(result.send_one_bool(a == b).await)
    }
}

/// Determine whether `a` is different from `b`
#[mel_treatment(
    input a Stream<bool>
    input b Stream<bool>
    output result Stream<bool>
)]
pub async fn not_equal() {
    while let (Ok(a), Ok(b)) = (a.recv_one_bool().await, b.recv_one_bool().await) {
        check!(result.send_one_bool(a != b).await)
    }
}

/// Makes _and_ ⋀ binary operation on `bool`
#[mel_function]
pub fn and(a: bool, b: bool) -> bool {
    a & b
}

/// Makes _or_ ⋁ binary operation on `bool`
#[mel_function]
pub fn or(a: bool, b: bool) -> bool {
    a | b
}

/// Makes _xor_ ⊕ binary operation on `bool`
#[mel_function]
pub fn xor(a: bool, b: bool) -> bool {
    a ^ b
}

/// Makes _not_ ¬ binary operation on `bool`
#[mel_function]
pub fn not(val: bool) -> bool {
    !val
}

/// Makes _and_ ⋀ binary operation on `bool`
#[mel_treatment(
    input a Stream<bool>
    input b Stream<bool>
    output result Stream<bool>
)]
pub async fn and() {
    while let (Ok(a), Ok(b)) = (a.recv_one_bool().await, b.recv_one_bool().await) {
        check!(result.send_one_bool(a & b).await)
    }
}

/// Makes _or_ ⋁ binary operation on `bool`
#[mel_treatment(
    input a Stream<bool>
    input b Stream<bool>
    output result Stream<bool>
)]
pub async fn or() {
    while let (Ok(a), Ok(b)) = (a.recv_one_bool().await, b.recv_one_bool().await) {
        check!(result.send_one_bool(a | b).await)
    }
}

/// Makes _xor_ ⊕ binary operation on `bool`
#[mel_treatment(
    input a Stream<bool>
    input b Stream<bool>
    output result Stream<bool>
)]
pub async fn xor() {
    while let (Ok(a), Ok(b)) = (a.recv_one_bool().await, b.recv_one_bool().await) {
        check!(result.send_one_bool(a ^ b).await)
    }
}

/// Makes _not_ ¬ binary operation on `bool`
#[mel_treatment(
    input value Stream<bool>
    output not Stream<bool>
)]
pub async fn not() {
    while let Ok(values) = value.recv_bool().await {
        check!(
            not.send_bool(values.into_iter().map(|v| !v).collect())
                .await
        )
    }
}
