use melodium_macro::{check, mel_treatment};
use melodium_core::*;

/// Flatten a stream of `Vec<i8>`.
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
    input vector Stream<Vec<i8>>
    output value Stream<i8>
)]
pub async fn flatten() {
    'main: while let Ok(vectors) = vector.recv_vec_i8().await {
        for vec in vectors {
            check!('main, value.send_i8(vec).await)
        }
    }
}

/// Chain two streams of `Vec<i8>`.
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
    input first Stream<Vec<i8>>
    input second Stream<Vec<i8>>
    output chained Stream<Vec<i8>>
)]
pub async fn chain() {

    while let Ok(vectors) = first.recv_vec_i8().await {

        check!(chained.send_vec_i8(vectors).await)
    }

    while let Ok(vectors) = second.recv_vec_i8().await {

        check!(chained.send_vec_i8(vectors).await)
    }
}

/// Merge two streams of `Vec<i8>`.
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
    input a Stream<Vec<i8>>
    input b Stream<Vec<i8>>
    input select Stream<bool>
    output value Stream<Vec<i8>>
)]
pub async fn merge() {
    while let Ok(select) = select.recv_one_bool().await {
        let val;
        if select {
            if let Ok(v) = a.recv_one_vec_i8().await {
                val = v;
            }
            else {
                break;
            }
        }
        else {
            if let Ok(v) = b.recv_one_vec_i8().await {
                val = v;
            }
            else {
                break;
            }
        }

        check!(value.send_one_vec_i8(val).await)
    }
}

/// Filter a `Vec<i8>` stream according to `bool` stream.
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
    input value Stream<Vec<i8>>
    input select Stream<bool>
    output accepted Stream<Vec<i8>>
    output rejected Stream<Vec<i8>>
)]
pub async fn filter() {

    let mut accepted_op = true;
    let mut rejected_op = true;

    while let (Ok(value), Ok(select)) = futures::join!(value.recv_one_vec_i8(), select.recv_one_bool()) {
        if select {
            if let Err(_) = accepted.send_one_vec_i8(value).await {
                // If we cannot send anymore on accepted, we note it,
                // and check if rejected is still valid, else just terminate.
                accepted_op = false;
                if !rejected_op {
                    break;
                }
            }
        }
        else {
            if let Err(_) = rejected.send_one_vec_i8(value).await {
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


/// Trigger on `Vec<i8>` stream start and end.
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
    input stream Stream<Vec<i8>>
    output start Block<void>
    output end Block<void>
    output first Block<Vec<i8>>
    output last Block<Vec<i8>>
)]
pub async fn trigger() {

    let mut last_value = None;

    if let Ok(values) = stream.recv_vec_i8().await {
        let _ = start.send_one_void(()).await;
        if let Some(val) = values.first().cloned() {
            let _ = first.send_one_vec_i8(val).await;
        }
        last_value = values.last().cloned();
        let _ = futures::join!(start.close(), first.close());
    }

    while let Ok(values) = stream.recv_vec_i8().await {
        last_value = values.last().cloned();
    }

    let _ = end.send_one_void(()).await;
    if let Some(val) = last_value {
        let _ = last.send_one_vec_i8(val).await;
    }

    // We don't close `end` and `last` explicitly here,
    // because it would be redundant with boilerplate
    // implementation of treatments.
}

/// Stream a block `Vec<i8>` element.
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
    input block Block<Vec<i8>>
    output stream Stream<Vec<i8>>
)]
pub async fn stream() {
    if let Ok(val) = block.recv_one_vec_i8().await {
        let _ = stream.send_one_vec_i8(val).await;
    }
}

/// Emit a block `Vec<i8>` value.
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
    output emit Block<Vec<i8>>
)]
pub async fn emit(value: Vec<i8>) {
    if let Ok(_) = trigger.recv_one_void().await {
        let _ = emit.send_one_vec_i8(value).await;
    }
}

/// Gives pattern of a `Vec<i8>` stream.
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
    input stream Stream<Vec<i8>>
    output pattern Stream<Vec<void>>
)]
pub async fn pattern() {
    while let Ok(vectors) = stream.recv_vec_i8().await {
        check!(pattern.send_vec_void(vectors.into_iter().map(|vec| vec![(); vec.len()]).collect()).await)
    }
}

/// Fit a stream of `i8` into stream of `Vec<i8>`, using a pattern.
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
    input value Stream<i8>
    input pattern Stream<Vec<void>>
    output fitted Stream<Vec<i8>>
)]
pub async fn fit() {
    'main: while let Ok(patterns) = pattern.recv_vec_void().await {
        for pattern in patterns {
            let mut vector = Vec::with_capacity(pattern.len());
            for _ in 0..pattern.len() {
                if let Ok(val) = value.recv_one_i8().await {
                    vector.push(val);
                }
                else {
                    // Uncomplete, we 'trash' vector
                    break 'main;
                }
            }
            check!('main, fitted.send_one_vec_i8(vector).await)
        }
    }
}

/// Fill a pattern stream with a `i8` value.
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
    default value 0
    input pattern Stream<Vec<void>>
    output filled Stream<Vec<i8>>
)]
pub async fn fill(value: i8) {
    while let Ok(pat) = pattern.recv_vec_void().await {
        check!(filled.send_vec_i8(pat.into_iter().map(|p| vec![value.clone(); p.len()]).collect()).await)
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
    default default 0
    input vector Stream<Vec<i8>>
    input size Stream<u64>
    output resized Stream<Vec<i8>>
)]
pub async fn resize(default: i8) {
    while let Ok(size) = size.recv_one_u64().await {
        if let Ok(mut vec) = vector.recv_one_vec_i8().await {
            vec.resize(size as usize, default.clone());
            check!(resized.send_one_vec_i8(vec).await);
        }
        else {
            break;
        }
    }
}
