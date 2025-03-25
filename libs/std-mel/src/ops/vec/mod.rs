use melodium_core::*;
use melodium_macro::{check, mel_function, mel_treatment};

pub mod block;

#[mel_function(
    generic T (PartialEquality)
)]
pub fn contains(vector: Vec<T>, value: T) -> bool {
    vector.iter().any(|val| val.partial_equality_eq(&value))
}

#[mel_treatment(
    generic T (PartialEquality)
    input value Stream<T>
    input vec Stream<Vec<T>>
    output contains Stream<bool>
)]
pub async fn contains() {
    while let (Ok(value), Ok(Value::Vec(vec))) = (value.recv_one().await, vec.recv_one().await) {
        check!(
            contains
                .send_one(vec.iter().any(|val| val.partial_equality_eq(&value)).into())
                .await
        )
    }
}

#[mel_function(
    generic T ()
)]
pub fn concat(mut first: Vec<T>, mut second: Vec<T>) -> Vec<T> {
    first.append(&mut second);
    first
}

#[mel_treatment(
    generic T ()
    input first Stream<Vec<T>>
    input second Stream<Vec<T>>
    output concatened Stream<Vec<T>>
)]
pub async fn concat() {
    while let (Ok(Value::Vec(mut first)), Ok(Value::Vec(mut second))) =
        (first.recv_one().await, second.recv_one().await)
    {
        first.append(&mut second);
        check!(concatened.send_one(Value::Vec(first)).await)
    }
}
