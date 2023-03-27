
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

/// Trigger on `void` stream start and end.
/// 
/// Emit `start` when a first value is send through the stream.
/// Emit `end` when stream is finally over.
/// 
/// Emit `first` with the first value coming in the stream.
/// Emit `last` with the last value coming in the stream.
/// 
/// ℹ️ `start` and `first` are always emitted together.
/// If the stream only contains one element, `first` and `last` both contains it.
/// If the stream never transmit any data before being ended, only `end` is emitted.
/// 
/// ```mermaid
/// graph LR
///     T("trigger()")
///     B["🟥 … 🟨 🟨 🟨 🟨 🟨 🟨 … 🟩"] -->|stream| T
///     
///     T -->|start| S["〈🟦〉"]
///     T -->|first| F["〈🟩〉"]
///     T -->|last| L["〈🟥〉"]
///     T -->|end| E["〈🟦〉"]
/// 
///     style B fill:#ffff,stroke:#ffff
///     style S fill:#ffff,stroke:#ffff
///     style F fill:#ffff,stroke:#ffff
///     style L fill:#ffff,stroke:#ffff
///     style E fill:#ffff,stroke:#ffff
/// ```
#[mel_treatment(
    input stream Stream<void>
    output start Block<void>
    output end Block<void>
    output first Block<void>
    output last Block<void>
)]
pub async fn trigger() {

    let mut last_value = None;

    if let Ok(values) = stream.recv_void().await {
        let _ = start.send_one_void(()).await;
        if let Some(val) = values.first().cloned() {
            let _ = first.send_one_void(val).await;
        }
        last_value = values.last().cloned();
        let _ = futures::join!(start.close(), first.close());
    }

    while let Ok(values) = stream.recv_void().await {
        last_value = values.last().cloned();
    }

    let _ = end.send_one_void(()).await;
    if let Some(val) = last_value {
        let _ = last.send_one_void(val).await;
    }

    // We don't close `end` and `last` explicitly here,
    // because it would be redundant with boilerplate
    // implementation of treatments.
}

/// Stream a block `void` value.
/// 
/// ```mermaid
/// graph LR
///     T("stream()")
///     B["〈🟦〉"] -->|block| T
///         
///     T -->|stream| S["🟦"]
///     
///     
///     style B fill:#ffff,stroke:#ffff
///     style S fill:#ffff,stroke:#ffff
/// ```
#[mel_treatment(
    input block Block<void>
    output stream Stream<void>
)]
pub async fn stream() {
    if let Ok(val) = block.recv_one_void().await {
        let _ = stream.send_one_void(val).await;
    }
}

/// Gives count of elements passing through stream.
/// 
/// This count increment one for each element within the stream, starting at 1.
/// 
/// ```mermaid
/// graph LR
///     T("count()")
///     V["🟦 🟦 🟦 …"] -->|iter| T
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

/// Generate a stream of `void` according to a length.
/// 
/// ```mermaid
/// graph LR
///     T("generate()")
///     B["〈🟨〉"] -->|length| T
///         
///     T -->|stream| S["… 🟦 🟦 🟦 🟦 🟦 🟦"]
///     
///     
///     style B fill:#ffff,stroke:#ffff
///     style S fill:#ffff,stroke:#ffff
/// ```
#[mel_treatment(
    input length Block<u128>
    output stream Stream<void>
)]
pub async fn generate() {

    if let Ok(length) = length.recv_one_u128().await {

        const CHUNK: u128 = 2u128.pow(20);
        let mut total = 0u128;
        while total < length {
            let chunk = u128::min(CHUNK, length - total) as usize;
            check!(stream.send_void(vec![(); chunk]).await);
            total += chunk as u128;
        }
    }
}

/// Generate a stream of `void` indefinitely.
/// 
/// This generates a continuous stream of `void`, until stream consumers closes it.
/// 
/// ```mermaid
/// graph LR
///     T("generateIndefinitely()")
///     B["〈🟦〉"] -->|trigger| T
///         
///     T -->|stream| S["… 🟦 🟦 🟦 🟦 🟦 🟦"]
///     
///     
///     style B fill:#ffff,stroke:#ffff
///     style S fill:#ffff,stroke:#ffff
/// ```
#[mel_treatment(
    input trigger Block<void>
    output stream Stream<void>
)]
pub async fn generate_indefinitely() {
    
    if let Ok(_) = trigger.recv_one_void().await {
        const CHUNK: usize = 2usize.pow(20);
        loop {
            check!(stream.send_void(vec![(); CHUNK]).await);
        }
    }
}
