
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

/// Chain two streams of `i8`.
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
    input first Stream<i8>
    input second Stream<i8>
    output chained Stream<i8>
)]
pub async fn chain() {

    while let Ok(values) = first.recv_i8().await {

        check!(chained.send_i8(values).await)
    }

    while let Ok(values) = second.recv_i8().await {

        check!(chained.send_i8(values).await)
    }
}
use melodium_macro::{check, mel_treatment};

/// Gives pattern of a `i8` stream.
/// 
/// ```mermaid
/// graph LR
///     T("pattern()")
///     A["… [🟨 🟨] [🟨] [🟨 🟨 🟨]"] -->|stream| T
///     
///     T -->|pattern| O["… [🟦 🟦] [🟦] [🟦 🟦 🟦]"]
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
///     B["[🟦 🟦] [🟦] [🟦 🟦 🟦]"] -->|pattern| T
///     
///     T -->|fitted| O["[🟨 🟨] [🟨] [🟨 🟨 🟨]"]
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

/// Merge two streams of `i8`.
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
    input a Stream<i8>
    input b Stream<i8>
    input select Stream<bool>
    output value Stream<i8>
)]
pub async fn merge() {
    while let Ok(select) = select.recv_one_bool().await {
        let val;
        if select {
            if let Ok(v) = a.recv_one_i8().await {
                val = v;
            }
            else {
                break;
            }
        }
        else {
            if let Ok(v) = b.recv_one_i8().await {
                val = v;
            }
            else {
                break;
            }
        }

        check!(value.send_one_i8(val).await)
    }
}
