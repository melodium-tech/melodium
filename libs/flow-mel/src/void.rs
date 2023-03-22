
use melodium_macro::{check, mel_treatment};
use melodium_core::*;

/// Chain two streams of `void`.
/// 
/// 
/// ```mermaid
/// graph LR
///     T("chain()")
///     A["🟨 🟨 🟨 🟨 🟨 🟨"] -->|first| T
///     B["… 🟪 🟪 🟪"] -->|second| T
///     
///     T -->|chained| O["… 🟪 🟪 🟪 🟨 🟨 🟨 🟨 🟨 🟨"]
/// 
///     style A fill:#ffff,stroke:#ffff
///     style B fill:#ffff,stroke:#ffff
///     style O fill:#ffff,stroke:#ffff
/// ```
#[mel_treatment(
    input first Stream<void>
    input second Stream<void>
    output chained Stream<void>
)]
pub async fn chain() {

    while let Ok(values) = first.recv_void().await {

        check!(chained.send_void(values).await)
    }

    while let Ok(values) = second.recv_void().await {

        check!(chained.send_void(values).await)
    }
}

/// Gives count of elements passing through stream.
/// 
/// This count increment one for each element within the stream, starting at 1.
/// 
/// ```mermaid
/// graph LR
///     T("count()")
///     V["🟦 🟦 🟦…"] -->|iter| T
///     
///     T -->|count| P["1️⃣ 2️⃣ 3️⃣ …"]
/// 
///     style V fill:#ffff,stroke:#ffff
///     style P fill:#ffff,stroke:#ffff
/// ```
#[mel_treatment(
    input stream Stream<void>
    output count Stream<u128>
)]
pub async fn count() {
    let mut i: u128 = 0;
    while let Ok(iter) = stream.recv_void().await {
        let next_i = i + iter.len() as u128;
        check!(count.send_u128((i..next_i).collect()).await);
        i = next_i;
    }
}