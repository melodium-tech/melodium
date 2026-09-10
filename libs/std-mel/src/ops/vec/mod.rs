use melodium_core::*;
use melodium_macro::{check, mel_function, mel_treatment};

pub mod block;

/// Return `true` if `vector` contains a value equal to `value`.
#[mel_function(
    generic T (PartialEquality)
)]
pub fn contains(vector: Vec<T>, value: T) -> bool {
    vector.iter().any(|val| val.partial_equality_eq(&value))
}

/// Pair-wise membership check over two streams.
///
/// For each (`value`, `vec`) pair received from the two streams, emit `true` through `contains` if `vec` contains `value`.
#[mel_treatment(
    generic T (PartialEquality)
    input value Stream<T>
    input vec Stream<Vec<T>>
    output contains Stream<bool>
)]
pub async fn contains() {
    while let (Ok(value), Ok(vec_value)) = (value.recv_one().await, vec.recv_one().await) {
        let vec = match vec_value {
            Value::Vec(vec) => vec,
            Value::Packed(arr) => arr.into_values(),
            _ => break,
        };
        check!(
            contains
                .send_one_as(vec.iter().any(|val| val.partial_equality_eq(&value)))
                .await
        )
    }
}

/// Concatenate `second` onto the end of `first` and return the combined vector.
#[mel_function(
    generic T ()
)]
pub fn concat(mut first: Vec<T>, mut second: Vec<T>) -> Vec<T> {
    first.append(&mut second);
    first
}

/// Pair-wise concatenation over two streams.
///
/// For each (`first`, `second`) pair received from the two streams, append `second` to `first` and emit the result through `concatened`.
#[mel_treatment(
    generic T ()
    input first Stream<Vec<T>>
    input second Stream<Vec<T>>
    output concatened Stream<Vec<T>>
)]
pub async fn concat() {
    while let (Ok(first_value), Ok(second_value)) =
        (first.recv_one().await, second.recv_one().await)
    {
        let mut first = match first_value {
            Value::Vec(vec) => vec,
            Value::Packed(arr) => arr.into_values(),
            _ => break,
        };
        let mut second = match second_value {
            Value::Vec(vec) => vec,
            Value::Packed(arr) => arr.into_values(),
            _ => break,
        };
        first.append(&mut second);
        check!(concatened.send_one_as(first).await)
    }
}
