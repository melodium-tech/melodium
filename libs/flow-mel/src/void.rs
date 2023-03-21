
/// Flatten a stream of `Vec<void>`.
/// 
/// All the input vectors are turned into continuous stream of scalar values, keeping order.
/// ```mermaid
/// graph LR
///     T("flatten()")
///     B["［🟦 🟦］［🟦］［🟦 🟦 🟦］"] -->|vector| T
///     
///     T -->|value| O["🟦 🟦 🟦 🟦 🟦 🟦"]
/// 
///     style B fill:#ffff,stroke:#ffff
///     style O fill:#ffff,stroke:#ffff
/// ```
#[mel_treatment(
    input vector Stream<Vec<void>>
    output value Stream<void>
)]
pub async fn flatten() {
    'main: while let Ok(vectors) = vector.recv_vec_void().await {
        for vec in vectors {
            check!('main, value.send_void(vec).await)
        }
    }
}

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