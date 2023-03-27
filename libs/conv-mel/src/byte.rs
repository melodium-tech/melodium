use melodium_macro::{check, mel_treatment};

/// Turns `byte` stream into `void` one.
#[mel_treatment(
    input value Stream<byte>
    output iter Stream<void>
)]
pub async fn to_void() {
    while let Ok(values) = value.recv_byte().await {
        check!(iter.send_void(vec![(); values.len()]).await)
    }
}

