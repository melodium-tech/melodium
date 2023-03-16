use melodium_core::*;
use melodium_macro::{check, mel_function, mel_treatment};

/// Turns `bool` stream into `void` one.
#[mel_treatment(
    input value Stream<bool>
    output iter Stream<void>
)]
pub async fn to_void() {
    while let Ok(values) = value.recv_bool().await {
        check!(iter.send_void(vec![(); values.len()]).await)
    }
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
    while let Ok(values) = value.recv_bool().await {
        check!(
            data.send_vec_byte(
                values
                    .into_iter()
                    .map(|val| vec![match val {
                        true => 1,
                        false => 0,
                    }])
                    .collect()
            )
            .await
        )
    }
}
