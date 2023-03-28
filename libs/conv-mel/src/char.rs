use melodium_core::*;
use melodium_macro::{check, mel_function, mel_treatment};

/// Turns `char` stream into `void` one.
#[mel_treatment(
    input value Stream<char>
    output iter Stream<void>
)]
pub async fn to_void() {
    while let Ok(values) = value.recv_char().await {
        check!(iter.send_void(vec![(); values.len()]).await)
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
    while let Ok(values) = value.recv_char().await {
        check!(
            data.send_vec_byte(
                values
                    .into_iter()
                    .map(|val| val.to_string().as_bytes().to_vec())
                    .collect()
            )
            .await
        )
    }
}
