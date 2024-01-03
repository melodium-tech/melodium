use melodium_core::*;
use melodium_macro::{check, mel_function, mel_treatment};

/// Turns `string` stream into `void` one.
#[mel_treatment(
    input value Stream<string>
    output iter Stream<void>
)]
pub async fn to_void() {
    while let Ok(values) = value.recv_many().await {
        check!(iter.send_many(vec![(); values.len()].into()).await)
    }
}

/// Turns `string` into `Vec<byte>`.
///
/// ℹ️ The `Vec<byte>` obtained from a `string` is the UTF-8 representation of it.
#[mel_function]
pub fn to_byte(value: string) -> Vec<byte> {
    value.as_bytes().to_vec()
}

/// Turns `string` stream into `byte` one.
///
/// Each `string` gets converted into `Vec<byte>`, with each vector containing the `byte`s of the former scalar `string` it represents.
///
/// ℹ️ The `Vec<byte>` obtained from a `string` is the UTF-8 representation of it.
#[mel_treatment(
    input value Stream<string>
    output data Stream<Vec<byte>>
)]
pub async fn to_byte() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| TryInto::<Vec<string>>::try_into(values).unwrap())
    {
        check!(
            data.send_many(TransmissionValue::Other(
                values
                    .into_iter()
                    .map(|val| Value::Vec(val.as_bytes().iter().map(|v| Value::Byte(*v)).collect()))
                    .collect()
            ))
            .await
        )
    }
}
