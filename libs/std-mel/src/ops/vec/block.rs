use melodium_core::*;
use melodium_macro::mel_treatment;

/// Check whether `vec` contains a value equal to `value` and emit the boolean result through `contains`.
#[mel_treatment(
    generic T (PartialEquality)
    input value Block<T>
    input vec Block<Vec<T>>
    output contains Block<bool>
)]
pub async fn contains() {
    if let (Ok(value), Ok(vec_value)) = (value.recv_one().await, vec.recv_one().await) {
        let vec = match vec_value {
            Value::Vec(vec) => vec,
            Value::Packed(arr) => arr.into_values(),
            _ => return,
        };
        let _ = contains
            .send_one_as(vec.iter().any(|val| val.partial_equality_eq(&value)))
            .await;
    }
}

/// Append the elements of `second` to `first` and emit the combined vector through `concatened`.
#[mel_treatment(
    generic T ()
    input first Block<Vec<T>>
    input second Block<Vec<T>>
    output concatened Block<Vec<T>>
)]
pub async fn concat() {
    if let (Ok(first_value), Ok(second_value)) = (first.recv_one().await, second.recv_one().await) {
        let mut first = match first_value {
            Value::Vec(vec) => vec,
            Value::Packed(arr) => arr.into_values(),
            _ => return,
        };
        let mut second = match second_value {
            Value::Vec(vec) => vec,
            Value::Packed(arr) => arr.into_values(),
            _ => return,
        };
        first.append(&mut second);
        let _ = concatened.send_one_as(first).await;
    }
}
