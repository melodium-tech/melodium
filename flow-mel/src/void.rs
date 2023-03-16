use melodium_core::*;
use melodium_macro::{check, mel_function, mel_treatment};

///
#[mel_treatment(
    input iter Stream<void>
    output count Stream<u128>
)]
pub async fn count() {
    let mut i: u128 = 0;
    while let Ok(iter) = iter.recv_void().await {
        let next_i = i + iter.len() as u128;
        check!(count.send_u128((i..next_i).collect()).await);
        i = next_i;
    }
}
