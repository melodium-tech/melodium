use melodium_core::*;
use melodium_macro::{check, mel_treatment};

/// Flatten a stream of `Vec<bool>`.
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
    input vector Stream<Vec<bool>>
    output value Stream<bool>
)]
pub async fn flatten() {
    'main: while let Ok(vectors) = vector.recv_vec_bool().await {
        for vec in vectors {
            check!('main, value.send_bool(vec).await)
        }
    }
}

/// Chain two streams of `Vec<bool>`.
///
///
/// ```mermaid
/// graph LR
///     T("chain()")
///     A["［🟨 🟨］［🟨 🟨 🟨］［🟨］"] -->|first| T
///     B["…［🟪］［🟪 🟪］"] -->|second| T
///     
///     T -->|chained| O["…［🟪］［🟪 🟪］［🟨 🟨］［🟨 🟨 🟨］［🟨］"]
///
///     style A fill:#ffff,stroke:#ffff
///     style B fill:#ffff,stroke:#ffff
///     style O fill:#ffff,stroke:#ffff
/// ```
#[mel_treatment(
    input first Stream<Vec<bool>>
    input second Stream<Vec<bool>>
    output chained Stream<Vec<bool>>
)]
pub async fn chain() {
    while let Ok(vectors) = first.recv_vec_bool().await {
        check!(chained.send_vec_bool(vectors).await)
    }

    while let Ok(vectors) = second.recv_vec_bool().await {
        check!(chained.send_vec_bool(vectors).await)
    }
}

/// Merge two streams of `Vec<bool>`.
///
/// The two streams are merged using the `select` stream:
/// - when `true`, vector from `a` is used;
/// - when `false`, vector from `b` is used.
///
/// ℹ️ No vector from either `a` or `b` are discarded, they are used when `select` give turn.
///
/// ⚠️ When `select` ends merge terminates without treating the remaining vectors from `a` and `b`.
/// When `select` give turn to `a` or `b` while the concerned stream is ended, the merge terminates.
/// Merge continues as long as `select` and concerned stream does, while the other can be ended.
///
/// ```mermaid
/// graph LR
///     T("merge()")
///     A["…［🟪 🟪 🟪］［🟪 🟪］…"] -->|a| T
///     B["…［🟨 🟨］［🟨］［🟨 🟨 🟨］…"] -->|b| T
///     O["… 🟩 🟥 🟥 🟩 🟥 …"] -->|select|T
///     
///
///     T -->|value| V["…［🟪 🟪 🟪］［🟨 🟨］［🟨］［🟪 🟪］［🟨 🟨 🟨］…"]
///
///     style V fill:#ffff,stroke:#ffff
///     style O fill:#ffff,stroke:#ffff
///     style A fill:#ffff,stroke:#ffff
///     style B fill:#ffff,stroke:#ffff
/// ```
#[mel_treatment(
    input a Stream<Vec<bool>>
    input b Stream<Vec<bool>>
    input select Stream<bool>
    output value Stream<Vec<bool>>
)]
pub async fn merge() {
    while let Ok(select) = select.recv_one_bool().await {
        let val;
        if select {
            if let Ok(v) = a.recv_one_vec_bool().await {
                val = v;
            } else {
                break;
            }
        } else {
            if let Ok(v) = b.recv_one_vec_bool().await {
                val = v;
            } else {
                break;
            }
        }

        check!(value.send_one_vec_bool(val).await)
    }
}

/// Filter a `Vec<bool>` stream according to `bool` stream.
///
/// ℹ️ If both streams are not the same size nothing is sent through accepted nor rejected.
///  
/// ```mermaid
/// graph LR
///     T("filter()")
///     V["…［🟪 🟪 🟪］［🟨 🟨］［🟨］［🟪 🟪］［🟨 🟨 🟨］…"] -->|value| T
///     D["… 🟩 🟥 🟥 🟩 🟥 …"] -->|select|T
///     
///     T -->|accepted| A["…［🟪 🟪 🟪］［🟪 🟪］…"]
///     T -->|rejected| R["…［🟨 🟨］［🟨］［🟨 🟨 🟨］…"]
///
///     style V fill:#ffff,stroke:#ffff
///     style D fill:#ffff,stroke:#ffff
///     style A fill:#ffff,stroke:#ffff
///     style R fill:#ffff,stroke:#ffff
/// ```
#[mel_treatment(
    input value Stream<Vec<bool>>
    input select Stream<bool>
    output accepted Stream<Vec<bool>>
    output rejected Stream<Vec<bool>>
)]
pub async fn filter() {
    let mut accepted_op = true;
    let mut rejected_op = true;

    while let (Ok(value), Ok(select)) =
        futures::join!(value.recv_one_vec_bool(), select.recv_one_bool())
    {
        if select {
            if let Err(_) = accepted.send_one_vec_bool(value).await {
                // If we cannot send anymore on accepted, we note it,
                // and check if rejected is still valid, else just terminate.
                accepted_op = false;
                if !rejected_op {
                    break;
                }
            }
        } else {
            if let Err(_) = rejected.send_one_vec_bool(value).await {
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

/// Trigger on `Vec<bool>` stream start and end.
///
/// Emit `start` when a first value is send through the stream.
/// Emit `end` when stream is finally over.
///
/// Emit `first` with the first vector coming in the stream.
/// Emit `last` with the last vector coming in the stream.
///
/// ℹ️ `start` and `first` are always emitted together.
/// If the stream only contains one vector, `first` and `last` both contains it.
/// If the stream never transmit any data before being ended, only `end` is emitted.
///
/// ```mermaid
/// graph LR
///     T("trigger()")
///     B["［🟥 🟥］ … ［🟨 🟨］ ［🟨 🟨］ ［🟨 🟨］ … ［🟩 🟩］"] -->|stream| T
///     
///     T -->|start| S["〈🟦〉"]
///     T -->|first| F["〈［🟩 🟩］〉"]
///     T -->|last| L["〈［🟥 🟥］〉"]
///     T -->|end| E["〈🟦〉"]
///
///     style B fill:#ffff,stroke:#ffff
///     style S fill:#ffff,stroke:#ffff
///     style F fill:#ffff,stroke:#ffff
///     style L fill:#ffff,stroke:#ffff
///     style E fill:#ffff,stroke:#ffff
/// ```
#[mel_treatment(
    input stream Stream<Vec<bool>>
    output start Block<void>
    output end Block<void>
    output first Block<Vec<bool>>
    output last Block<Vec<bool>>
)]
pub async fn trigger() {
    let mut last_value = None;

    if let Ok(values) = stream.recv_vec_bool().await {
        let _ = start.send_one_void(()).await;
        if let Some(val) = values.first().cloned() {
            let _ = first.send_one_vec_bool(val).await;
        }
        last_value = values.last().cloned();
        let _ = futures::join!(start.close(), first.close());
    }

    while let Ok(values) = stream.recv_vec_bool().await {
        last_value = values.last().cloned();
    }

    let _ = end.send_one_void(()).await;
    if let Some(val) = last_value {
        let _ = last.send_one_vec_bool(val).await;
    }

    // We don't close `end` and `last` explicitly here,
    // because it would be redundant with boilerplate
    // implementation of treatments.
}

/// Stream a block `Vec<bool>` element.
///
/// ```mermaid
/// graph LR
///     T("stream()")
///     B["〈［🟦］〉"] -->|block| T
///         
///     T -->|stream| S["［🟦］"]
///     
///     
///     style B fill:#ffff,stroke:#ffff
///     style S fill:#ffff,stroke:#ffff
/// ```
#[mel_treatment(
    input block Block<Vec<bool>>
    output stream Stream<Vec<bool>>
)]
pub async fn stream() {
    if let Ok(val) = block.recv_one_vec_bool().await {
        let _ = stream.send_one_vec_bool(val).await;
    }
}

/// Emit a block `Vec<bool>` value.
///
/// When `trigger` is enabled, `value` is emitted as block.
///
/// ```mermaid
/// graph LR
///     T("emit(value=［🟨］)")
///     B["〈🟦〉"] -->|trigger| T
///         
///     T -->|emit| S["〈［🟨］〉"]
///     
///     style B fill:#ffff,stroke:#ffff
///     style S fill:#ffff,stroke:#ffff
/// ```
#[mel_treatment(
    input trigger Block<void>
    output emit Block<Vec<bool>>
)]
pub async fn emit(value: Vec<bool>) {
    if let Ok(_) = trigger.recv_one_void().await {
        let _ = emit.send_one_vec_bool(value).await;
    }
}

/// Gives pattern of a `Vec<bool>` stream.
///
/// ```mermaid
/// graph LR
///     T("pattern()")
///     A["…［🟨 🟨］［🟨］［🟨 🟨 🟨］"] -->|stream| T
///     
///     T -->|pattern| O["… ［🟦 🟦］［🟦］［🟦 🟦 🟦］"]
///
///     style A fill:#ffff,stroke:#ffff
///     style O fill:#ffff,stroke:#ffff
/// ```
#[mel_treatment(
    input stream Stream<Vec<bool>>
    output pattern Stream<Vec<void>>
)]
pub async fn pattern() {
    while let Ok(vectors) = stream.recv_vec_bool().await {
        check!(
            pattern
                .send_vec_void(vectors.into_iter().map(|vec| vec![(); vec.len()]).collect())
                .await
        )
    }
}

/// Fit a stream of `bool` into stream of `Vec<bool>`, using a pattern.
///
/// ℹ️ If some remaining values doesn't fit into the pattern, they are trashed.
/// If there are not enough values to fit the pattern, uncomplete vector is trashed.
///
/// ```mermaid
/// graph LR
///     T("fit()")
///     A["… 🟨 🟨 🟨 🟨 🟨 🟨"] -->|value| T
///     B["［🟦 🟦］［🟦］［🟦 🟦 🟦］"] -->|pattern| T
///     
///     T -->|fitted| O["［🟨 🟨］［🟨］［🟨 🟨 🟨］"]
///
///     style A fill:#ffff,stroke:#ffff
///     style B fill:#ffff,stroke:#ffff
///     style O fill:#ffff,stroke:#ffff
/// ```
#[mel_treatment(
    input value Stream<bool>
    input pattern Stream<Vec<void>>
    output fitted Stream<Vec<bool>>
)]
pub async fn fit() {
    'main: while let Ok(patterns) = pattern.recv_vec_void().await {
        for pattern in patterns {
            let mut vector = Vec::with_capacity(pattern.len());
            for _ in 0..pattern.len() {
                if let Ok(val) = value.recv_one_bool().await {
                    vector.push(val);
                } else {
                    // Uncomplete, we 'trash' vector
                    break 'main;
                }
            }
            check!('main, fitted.send_one_vec_bool(vector).await)
        }
    }
}

/// Fill a pattern stream with a `bool` value.
///
/// ```mermaid
/// graph LR
/// T("fill(value=🟧)")
/// B["…［🟦 🟦］［🟦］［🟦 🟦 🟦］…"] -->|pattern| T
///
/// T -->|filled| O["…［🟧 🟧］［🟧］［🟧 🟧 🟧］…"]
///
/// style B fill:#ffff,stroke:#ffff
/// style O fill:#ffff,stroke:#ffff
/// ```
#[mel_treatment(
    default value false
    input pattern Stream<Vec<void>>
    output filled Stream<Vec<bool>>
)]
pub async fn fill(value: bool) {
    while let Ok(pat) = pattern.recv_vec_void().await {
        check!(
            filled
                .send_vec_bool(
                    pat.into_iter()
                        .map(|p| vec![value.clone(); p.len()])
                        .collect()
                )
                .await
        )
    }
}

/// Resize vectors according to given streamed size.
///
/// If a vector is smaller than expected size, it is extended using the `default` value.
///
/// ```mermaid
/// graph LR
///     T("resize(default=🟨)")
///     V["［🟦 🟦］［🟦］［］［🟦 🟦 🟦］…"] -->|vector| T
///     S["3️⃣ 2️⃣ 3️⃣ 2️⃣ …"] -->|size| T
///     
///     T -->|resized| P["［🟦 🟦 🟨］［🟦 🟨］［🟨 🟨 🟨］［🟦 🟦］…"]
///
///     style V fill:#ffff,stroke:#ffff
///     style S fill:#ffff,stroke:#ffff
///     style P fill:#ffff,stroke:#ffff
/// ```
#[mel_treatment(
    default default false
    input vector Stream<Vec<bool>>
    input size Stream<u64>
    output resized Stream<Vec<bool>>
)]
pub async fn resize(default: bool) {
    while let Ok(size) = size.recv_one_u64().await {
        if let Ok(mut vec) = vector.recv_one_vec_bool().await {
            vec.resize(size as usize, default.clone());
            check!(resized.send_one_vec_bool(vec).await);
        } else {
            break;
        }
    }
}
