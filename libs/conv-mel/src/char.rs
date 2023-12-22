use melodium_core::*;
use melodium_macro::{check, mel_function, mel_treatment};

/// Turns `char` stream into `void` one.
#[mel_treatment(
    input value Stream<char>
    output iter Stream<void>
)]
pub async fn to_void() {
    while let Ok(values) = value.recv_many().await {
        check!(iter.send_many(vec![(); values.len()].into()).await)
    }
}

/// Turns `char` into `Vec<byte>`.
///
/// ℹ️ The `Vec<byte>` obtained from a `char` is the UTF-8 representation of it, the vector length is always 1 ≤ len ≤ 4.
#[mel_function]
pub fn to_byte(value: char) -> Vec<byte> {
    value.to_string().as_bytes().to_vec()
}

/// Turns `char` stream into `byte` one.
///
/// Each `char` gets converted into `Vec<byte>`, with each vector containing the `byte`s of the former scalar `char` it represents.
///
/// ℹ️ The `Vec<byte>` obtained from a `char` is the UTF-8 representation of it, the vector length is always 1 ≤ len ≤ 4.
#[mel_treatment(
    input value Stream<char>
    output data Stream<Vec<byte>>
)]
pub async fn to_byte() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| TryInto::<Vec<char>>::try_into(values).unwrap())
    {
        check!(
            data.send_many(TransmissionValue::Other(
                values
                    .into_iter()
                    .map(|val| Value::Vec(
                        val.to_string()
                            .as_bytes()
                            .iter()
                            .map(|v| Value::Byte(*v))
                            .collect()
                    ))
                    .collect()
            ))
            .await
        )
    }
}
