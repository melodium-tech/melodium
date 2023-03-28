use melodium_core::*;
use melodium_macro::{check, mel_treatment};

/// Chain two streams of `char`.
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
    input first Stream<char>
    input second Stream<char>
    output chained Stream<char>
)]
pub async fn chain() {
    while let Ok(values) = first.recv_char().await {
        check!(chained.send_char(values).await)
    }

    while let Ok(values) = second.recv_char().await {
        check!(chained.send_char(values).await)
    }
}

/// Trigger on `char` stream start and end.
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
    input stream Stream<char>
    output start Block<void>
    output end Block<void>
    output first Block<char>
    output last Block<char>
)]
pub async fn trigger() {
    let mut last_value = None;

    if let Ok(values) = stream.recv_char().await {
        let _ = start.send_one_void(()).await;
        if let Some(val) = values.first().cloned() {
            let _ = first.send_one_char(val).await;
        }
        last_value = values.last().cloned();
        let _ = futures::join!(start.close(), first.close());
    }

    while let Ok(values) = stream.recv_char().await {
        last_value = values.last().cloned();
    }

    let _ = end.send_one_void(()).await;
    if let Some(val) = last_value {
        let _ = last.send_one_char(val).await;
    }

    // We don't close `end` and `last` explicitly here,
    // because it would be redundant with boilerplate
    // implementation of treatments.
}

/// Stream a block `char` value.
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
    input block Block<char>
    output stream Stream<char>
)]
pub async fn stream() {
    if let Ok(val) = block.recv_one_char().await {
        let _ = stream.send_one_char(val).await;
    }
}

/// Merge two streams of `char`.
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
    input a Stream<char>
    input b Stream<char>
    input select Stream<bool>
    output value Stream<char>
)]
pub async fn merge() {
    while let Ok(select) = select.recv_one_bool().await {
        let val;
        if select {
            if let Ok(v) = a.recv_one_char().await {
                val = v;
            } else {
                break;
            }
        } else {
            if let Ok(v) = b.recv_one_char().await {
                val = v;
            } else {
                break;
            }
        }

        check!(value.send_one_char(val).await)
    }
}

/// Fill a pattern stream with a `char` value.
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
    default value '∅'
    input pattern Stream<void>
    output filled Stream<char>
)]
pub async fn fill(value: char) {
    while let Ok(pat) = pattern.recv_void().await {
        check!(filled.send_char(vec![value.clone(); pat.len()]).await)
    }
}

/// Filter a `char` stream according to `bool` stream.
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
    input value Stream<char>
    input select Stream<bool>
    output accepted Stream<char>
    output rejected Stream<char>
)]
pub async fn filter() {
    let mut accepted_op = true;
    let mut rejected_op = true;

    while let (Ok(value), Ok(select)) =
        futures::join!(value.recv_one_char(), select.recv_one_bool())
    {
        if select {
            if let Err(_) = accepted.send_one_char(value).await {
                // If we cannot send anymore on accepted, we note it,
                // and check if rejected is still valid, else just terminate.
                accepted_op = false;
                if !rejected_op {
                    break;
                }
            }
        } else {
            if let Err(_) = rejected.send_one_char(value).await {
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

/// Fit a stream of `char` into a pattern.
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
    input value Stream<char>
    input pattern Stream<void>
    output fitted Stream<char>
)]
pub async fn fit() {
    'main: while let Ok(pattern) = pattern.recv_void().await {
        for _ in pattern {
            if let Ok(val) = value.recv_one_char().await {
                check!('main, fitted.send_one_char(val).await)
            } else {
                break 'main;
            }
        }
    }
}

/// Emit a block `char` value.
///
/// When `trigger` is enabled, `value` is emitted as block.
///
/// ```mermaid
/// graph LR
///     T("emit(value=🟨)")
///     B["〈🟦〉"] -->|trigger| T
///         
///     T -->|emit| S["〈🟨〉"]
///     
///     style B fill:#ffff,stroke:#ffff
///     style S fill:#ffff,stroke:#ffff
/// ```
#[mel_treatment(
    input trigger Block<void>
    output emit Block<char>
)]
pub async fn emit(value: char) {
    if let Ok(_) = trigger.recv_one_void().await {
        let _ = emit.send_one_char(value).await;
    }
}
