
use melodium_macro::{check, mel_treatment};
use melodium_core::*;

/// Chain two streams of `i16`.
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
    input first Stream<i16>
    input second Stream<i16>
    output chained Stream<i16>
)]
pub async fn chain() {

    while let Ok(values) = first.recv_i16().await {

        check!(chained.send_i16(values).await)
    }

    while let Ok(values) = second.recv_i16().await {

        check!(chained.send_i16(values).await)
    }
}

/// Merge two streams of `i16`.
/// 
/// The two streams are merged using the `select` stream:
/// - when `true`, value from `a` is used;
/// - when `false`, value from `b` is used.
/// 
/// ℹ️ No value from either `a` or `b` are discarded, they are used when `select` give turn.
/// 
/// ⚠️ When `select` ends merge terminates without treating the remaining values from `a` and `b`.
/// When `select` give turn to `a` or `b` while the concerned stream is ended, the merge terminates.
/// Merge continues as long as `select` and concerned stream does, while the other can be ended.
/// 
/// ```mermaid
/// graph LR
///     T("merge()")
///     A["… 🟦 🟫 …"] -->|a| T
///     B["… 🟧 🟪 🟨 …"] -->|b| T
///     O["… 🟩 🟥 🟥 🟩 🟥 …"] -->|select|T
///     
/// 
///     T -->|value| V["… 🟦 🟧 🟪 🟫 🟨 …"]
/// 
///     style V fill:#ffff,stroke:#ffff
///     style O fill:#ffff,stroke:#ffff
///     style A fill:#ffff,stroke:#ffff
///     style B fill:#ffff,stroke:#ffff
/// ```
#[mel_treatment(
    input a Stream<i16>
    input b Stream<i16>
    input select Stream<bool>
    output value Stream<i16>
)]
pub async fn merge() {
    while let Ok(select) = select.recv_one_bool().await {
        let val;
        if select {
            if let Ok(v) = a.recv_one_i16().await {
                val = v;
            }
            else {
                break;
            }
        }
        else {
            if let Ok(v) = b.recv_one_i16().await {
                val = v;
            }
            else {
                break;
            }
        }

        check!(value.send_one_i16(val).await)
    }
}

/// Fill a pattern stream with a `i16` value.
/// 
/// ```mermaid
/// graph LR
/// T("fill(value=🟧)")
/// B["… 🟦 🟦 🟦 …"] -->|pattern| T
/// 
/// T -->|filled| O["… 🟧 🟧 🟧 …"]
/// 
/// style B fill:#ffff,stroke:#ffff
/// style O fill:#ffff,stroke:#ffff
/// ```
#[mel_treatment(
    default value 0
    input pattern Stream<void>
    output filled Stream<i16>
)]
pub async fn fill(value: i16) {
    while let Ok(pat) = pattern.recv_void().await {
        check!(filled.send_i16(vec![value.clone(); pat.len()]).await)
    }
}

/// Filter a `i16` stream according to `bool` stream.
/// 
/// ℹ️ If both streams are not the same size nothing is sent through accepted nor rejected.
///  
/// ```mermaid
/// graph LR
///     T("filter()")
///     V["… 🟦 🟧 🟪 🟫 🟨 …"] -->|value| T
///     D["… 🟩 🟥 🟥 🟩 🟥 …"] -->|select|T
///     
///     T -->|accepted| A["… 🟦 🟫 …"]
///     T -->|rejected| R["… 🟧 🟪 🟨 …"]
/// 
///     style V fill:#ffff,stroke:#ffff
///     style D fill:#ffff,stroke:#ffff
///     style A fill:#ffff,stroke:#ffff
///     style R fill:#ffff,stroke:#ffff
/// ```
#[mel_treatment(
    input value Stream<i16>
    input select Stream<bool>
    output accepted Stream<i16>
    output rejected Stream<i16>
)]
pub async fn filter() {

    let mut accepted_op = true;
    let mut rejected_op = true;

    while let (Ok(value), Ok(select)) = futures::join!(value.recv_one_i16(), select.recv_one_bool()) {
        if select {
            if let Err(_) = accepted.send_one_i16(value).await {
                // If we cannot send anymore on accepted, we note it,
                // and check if rejected is still valid, else just terminate.
                accepted_op = false;
                if !rejected_op {
                    break;
                }
            }
        }
        else {
            if let Err(_) = rejected.send_one_i16(value).await {
                // If we cannot send anymore on rejected, we note it,
                // and check if accepted is still valid, else just terminate.
                rejected_op = false;
                if !accepted_op {
                    break;
                }
            }
        }
    }
}

/// Fit a stream of `i16` into a pattern.
/// 
/// ℹ️ If some remaining values doesn't fit into the pattern, they are trashed.
/// 
/// ```mermaid
/// graph LR
///     T("fit()")
///     A["… 🟨 🟨 🟨 🟨 🟨 🟨"] -->|value| T
///     B["🟦 🟦 🟦 🟦"] -->|pattern| T
///     
///     T -->|fitted| O["🟨 🟨 🟨 🟨"]
/// 
///     style A fill:#ffff,stroke:#ffff
///     style B fill:#ffff,stroke:#ffff
///     style O fill:#ffff,stroke:#ffff
/// ```
#[mel_treatment(
    input value Stream<i16>
    input pattern Stream<void>
    output fitted Stream<i16>
)]
pub async fn fit() {
    'main: while let Ok(pattern) = pattern.recv_void().await {
        for _ in pattern {
            if let Ok(val) = value.recv_one_i16().await {
                check!('main, fitted.send_one_i16(val).await)
            }
            else {
                break 'main;
            }
        }
    }
}
