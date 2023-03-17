use melodium_macro::{check, mel_treatment};
use melodium_core::*;

/// Gives count of elements passing through stream
/// 
/// This count increment one for each elements within the `iter` stream, starting at 1.
/// 
/// ```mermaid
/// graph LR
///     T("Count()")
///     V["🟦 🟦 🟦…"] -->|iter| T
///     
///     T -->|count| P["1️⃣ 2️⃣ 3️⃣ …"]
/// 
///     style V fill:#ffff,stroke:#ffff
///     style P fill:#ffff,stroke:#ffff
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