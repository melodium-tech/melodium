use melodium_macro::{check, mel_function, mel_treatment};
use melodium_core::*;

/// Turns `string` stream into `void` one.
#[mel_treatment(
    input value Stream<string>
    output iter Stream<void>
)]
pub async fn to_void() {
    while let Ok(values) = value.recv_string().await {
        check!(iter.send_void(vec![(); values.len()]).await)
    }
}


/// Turns `string` stream into `byte` one.
/// 
/// Each `string` gets converted into `Vec<byte>`, with each vector containing the `byte`s of the former scalar `string` it represents.
#[mel_treatment(
    input value Stream<string>
    output data Stream<Vec<byte>>
)]
pub async fn to_byte() {
    while let Ok(values) = value.recv_string().await {
        check!(data.send_vec_byte(values.into_iter().map(|val| val.as_bytes().to_vec()).collect()).await)
    }
}
