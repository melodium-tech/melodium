use melodium_core::*;
use melodium_macro::{check, mel_function, mel_treatment};

/// Return whether `a` is equal to `b`
#[mel_function]
pub fn equal(a: byte, b: byte) -> bool {
    a == b
}

/// Return whether `a` is different `b`
#[mel_function]
pub fn not_equal(a: byte, b: byte) -> bool {
    a != b
}

/// Determine whether `a` is equal to `b`
#[mel_treatment(
    input a Stream<byte>
    input b Stream<byte>
    output result Stream<bool>
)]
pub async fn equal() {
    while let (Ok(a), Ok(b)) = (
        a.recv_one()
            .await
            .map(|val| GetData::<byte>::try_data(val).unwrap()),
        b.recv_one()
            .await
            .map(|val| GetData::<byte>::try_data(val).unwrap()),
    ) {
        check!(result.send_one((a == b).into()).await)
    }
}

/// Determine whether `a` is different from `b`
#[mel_treatment(
    input a Stream<byte>
    input b Stream<byte>
    output result Stream<bool>
)]
pub async fn not_equal() {
    while let (Ok(a), Ok(b)) = (
        a.recv_one()
            .await
            .map(|val| GetData::<byte>::try_data(val).unwrap()),
        b.recv_one()
            .await
            .map(|val| GetData::<byte>::try_data(val).unwrap()),
    ) {
        check!(result.send_one((a != b).into()).await)
    }
}

/// Makes _and_ ⋀ binary operation on `byte`
#[mel_function]
pub fn and(a: byte, b: byte) -> byte {
    a & b
}

/// Makes _or_ ⋁ binary operation on `byte`
#[mel_function]
pub fn or(a: byte, b: byte) -> byte {
    a | b
}

/// Makes _xor_ ⊕ binary operation on `byte`
#[mel_function]
pub fn xor(a: byte, b: byte) -> byte {
    a ^ b
}

/// Makes _not_ ¬ binary operation on `byte`
#[mel_function]
pub fn not(val: byte) -> byte {
    !val
}

/// Makes _and_ ⋀ binary operation on `byte`
#[mel_treatment(
    input a Stream<byte>
    input b Stream<byte>
    output result Stream<byte>
)]
pub async fn and() {
    while let (Ok(a), Ok(b)) = (
        a.recv_one()
            .await
            .map(|val| GetData::<byte>::try_data(val).unwrap()),
        b.recv_one()
            .await
            .map(|val| GetData::<byte>::try_data(val).unwrap()),
    ) {
        check!(result.send_one((a & b).into()).await)
    }
}

/// Makes _or_ ⋁ binary operation on `byte`
#[mel_treatment(
    input a Stream<byte>
    input b Stream<byte>
    output result Stream<byte>
)]
pub async fn or() {
    while let (Ok(a), Ok(b)) = (
        a.recv_one()
            .await
            .map(|val| GetData::<byte>::try_data(val).unwrap()),
        b.recv_one()
            .await
            .map(|val| GetData::<byte>::try_data(val).unwrap()),
    ) {
        check!(result.send_one((a | b).into()).await)
    }
}

/// Makes _xor_ ⊕ binary operation on `byte`
#[mel_treatment(
    input a Stream<byte>
    input b Stream<byte>
    output result Stream<byte>
)]
pub async fn xor() {
    while let (Ok(a), Ok(b)) = (
        a.recv_one()
            .await
            .map(|val| GetData::<byte>::try_data(val).unwrap()),
        b.recv_one()
            .await
            .map(|val| GetData::<byte>::try_data(val).unwrap()),
    ) {
        check!(result.send_one((a ^ b).into()).await)
    }
}

/// Makes _not_ ¬ binary operation on `byte`
#[mel_treatment(
    input value Stream<byte>
    output not Stream<byte>
)]
pub async fn not() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| TryInto::<Vec<byte>>::try_into(values).unwrap())
    {
        check!(
            not.send_many(
                values
                    .into_iter()
                    .map(|v| !v)
                    .collect::<VecDeque<_>>()
                    .into()
            )
            .await
        )
    }
}
