use melodium_macro::{check, mel_treatment};
use melodium_core::*;

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

/// Chain two streams of `Vec<void>`.
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
    input first Stream<Vec<void>>
    input second Stream<Vec<void>>
    output chained Stream<Vec<void>>
)]
pub async fn chain() {

    while let Ok(vectors) = first.recv_vec_void().await {

        check!(chained.send_vec_void(vectors).await)
    }

    while let Ok(vectors) = second.recv_vec_void().await {

        check!(chained.send_vec_void(vectors).await)
    }
}

/// Merge two streams of `Vec<void>`.
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
    input a Stream<Vec<void>>
    input b Stream<Vec<void>>
    input select Stream<bool>
    output value Stream<Vec<void>>
)]
pub async fn merge() {
    while let Ok(select) = select.recv_one_bool().await {
        let val;
        if select {
            if let Ok(v) = a.recv_one_vec_void().await {
                val = v;
            }
            else {
                break;
            }
        }
        else {
            if let Ok(v) = b.recv_one_vec_void().await {
                val = v;
            }
            else {
                break;
            }
        }

        check!(value.send_one_vec_void(val).await)
    }
}

/// Filter a `Vec<void>` stream according to `bool` stream.
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
    input value Stream<Vec<void>>
    input select Stream<bool>
    output accepted Stream<Vec<void>>
    output rejected Stream<Vec<void>>
)]
pub async fn filter() {

    let mut accepted_op = true;
    let mut rejected_op = true;

    while let (Ok(value), Ok(select)) = futures::join!(value.recv_one_vec_void(), select.recv_one_bool()) {
        if select {
            if let Err(_) = accepted.send_one_vec_void(value).await {
                // If we cannot send anymore on accepted, we note it,
                // and check if rejected is still valid, else just terminate.
                accepted_op = false;
                if !rejected_op {
                    break;
                }
            }
        }
        else {
            if let Err(_) = rejected.send_one_vec_void(value).await {
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

/// Gives count of elements passing through stream.
/// 
/// This count increment one for each vector within the stream, starting at 1.
/// ℹ️ The count is independant from vector sizes.
/// 
/// ```mermaid
/// graph LR
///     T("count()")
///     V["［🟦 🟦］［🟦］［🟦 🟦 🟦］…"] -->|stream| T
///     
///     T -->|count| P["1️⃣ 2️⃣ 3️⃣ …"]
/// 
///     style V fill:#ffff,stroke:#ffff
///     style P fill:#ffff,stroke:#ffff
/// ```
#[mel_treatment(
    input stream Stream<Vec<void>>
    output count Stream<u128>
)]
pub async fn count() {
    let mut i: u128 = 0;
    while let Ok(iter) = stream.recv_vec_void().await {
        let next_i = i + iter.len() as u128;
        check!(count.send_u128((i..next_i).collect()).await);
        i = next_i;
    }
}

/// Gives size of vectors passing through stream.
/// 
/// For each vector one `size` value is sent, giving the number of elements contained within matching vector.
/// 
/// ```mermaid
/// graph LR
///     T("size()")
///     V["［🟦 🟦］［🟦］［］［🟦 🟦 🟦］…"] -->|vector| T
///     
///     T -->|size| P["2️⃣ 1️⃣ 0️⃣ 3️⃣ …"]
/// 
///     style V fill:#ffff,stroke:#ffff
///     style P fill:#ffff,stroke:#ffff
/// ```
#[mel_treatment(
    input vector Stream<Vec<void>>
    output size Stream<u64>
)]
pub async fn size() {
    while let Ok(iter) = vector.recv_vec_void().await {
        check!(size.send_u64(iter.into_iter().map(|v| v.len() as u64).collect()).await);
    }
}
