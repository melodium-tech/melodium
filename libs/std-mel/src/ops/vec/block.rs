use melodium_core::*;
use melodium_macro::mel_treatment;

#[mel_treatment(
    generic T (PartialEquality)
    input value Block<T>
    input vec Block<Vec<T>>
    output contains Block<bool>
)]
pub async fn contains() {
    if let (Ok(value), Ok(Value::Vec(vec))) = (value.recv_one().await, vec.recv_one().await) {
        let _ = contains
            .send_one(vec.iter().any(|val| val.partial_equality_eq(&value)).into())
            .await;
    }
}

#[mel_treatment(
    generic T ()
    input first Block<Vec<T>>
    input second Block<Vec<T>>
    output concatened Block<Vec<T>>
)]
pub async fn concat() {
    if let (Ok(Value::Vec(mut first)), Ok(Value::Vec(mut second))) =
        (first.recv_one().await, second.recv_one().await)
    {
        first.append(&mut second);
        let _ = concatened.send_one(Value::Vec(first)).await;
    }
}
