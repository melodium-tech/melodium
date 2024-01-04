use melodium_core::*;
use melodium_macro::{check, mel_function, mel_treatment};

/// Turns `bool` stream into `void` one.
#[mel_treatment(
    input value Stream<bool>
    output iter Stream<void>
)]
pub async fn to_void() {
    while let Ok(values) = value.recv_many().await {
        check!(iter.send_many(vec![(); values.len()].into()).await)
    }
}

/// Turns `bool` into `Vec<byte>`.
///
/// ℹ️ A `bool` always corresponds to one `byte`, being `0` if `false` and `1` if `true`.
#[mel_function]
pub fn to_byte(value: bool) -> Vec<byte> {
    vec![match value {
        true => 1,
        false => 0,
    }]
}

/// Turns `bool` stream into `byte` one.
///
/// Each `bool` gets converted into `Vec<byte>`, with each vector containing the `byte` of the former scalar `bool` it represents.
///
/// ℹ️ A `bool` always corresponds to one `byte`, being `0` if `false` and `1` if `true`.
#[mel_treatment(
    input value Stream<bool>
    output data Stream<Vec<byte>>
)]
pub async fn to_byte() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| TryInto::<Vec<bool>>::try_into(values).unwrap())
    {
        check!(
            data.send_many(TransmissionValue::Other(
                values
                    .into_iter()
                    .map(|val| {
                        Value::Vec(vec![match val {
                            true => Value::Byte(1),
                            false => Value::Byte(0),
                        }])
                    })
                    .collect(),
            ))
            .await
        )
    }
}
