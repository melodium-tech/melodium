use melodium_core::*;
use melodium_macro::{check, mel_function, mel_treatment};

/// Return whether `a` is equal to `b`
#[mel_function]
pub fn equal(a: string, b: string) -> bool {
    a == b
}

/// Return whether `a` is different `b`
#[mel_function]
pub fn not_equal(a: string, b: string) -> bool {
    a != b
}

/// Determine whether `a` is equal to `b`
#[mel_treatment(
    input a Stream<string>
    input b Stream<string>
    output result Stream<bool>
)]
pub async fn equal() {
    while let (Ok(a), Ok(b)) = (
        a.recv_one()
            .await
            .map(|val| GetData::<string>::try_data(val).unwrap()),
        b.recv_one()
            .await
            .map(|val| GetData::<string>::try_data(val).unwrap()),
    ) {
        check!(result.send_one((a == b).into()).await)
    }
}

/// Determine whether `a` is different from `b`
#[mel_treatment(
    input a Stream<string>
    input b Stream<string>
    output result Stream<bool>
)]
pub async fn not_equal() {
    while let (Ok(a), Ok(b)) = (
        a.recv_one()
            .await
            .map(|val| GetData::<string>::try_data(val).unwrap()),
        b.recv_one()
            .await
            .map(|val| GetData::<string>::try_data(val).unwrap()),
    ) {
        check!(result.send_one((a != b).into()).await)
    }
}
