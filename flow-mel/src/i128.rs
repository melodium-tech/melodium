use melodium_core::*;
use melodium_macro::{check, mel_function, mel_treatment};

/// Gives pattern of a `i128` stream.
///
/// ```mermaid
/// graph LR
///     T("pattern()")
///     A["… [🟨 🟨] [🟨] [🟨 🟨 🟨]"] -->|stream| T
///     
///     T -->|fitted| O["… [🟦 🟦] [🟦] [🟦 🟦 🟦]"]
///
///     style A fill:#ffff,stroke:#ffff
///     style O fill:#ffff,stroke:#ffff
/// ```
#[mel_treatment(
    input stream Stream<Vec<i128>>
    output pattern Stream<Vec<void>>
)]
pub async fn pattern() {
    while let Ok(vectors) = stream.recv_vec_i128().await {
        check!(
            pattern
                .send_vec_void(vectors.into_iter().map(|vec| vec![(); vec.len()]).collect())
                .await
        )
    }
}

/// Fit a stream of `i128` into stream of `Vec<i128>`, using a pattern.
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
    input value Stream<i128>
    input pattern Stream<Vec<void>>
    output fitted Stream<Vec<i128>>
)]
pub async fn fit() {
    'main: while let Ok(patterns) = pattern.recv_vec_void().await {
        for pattern in patterns {
            let mut vector = Vec::with_capacity(pattern.len());
            for _ in 0..pattern.len() {
                if let Ok(val) = value.recv_one_i128().await {
                    vector.push(val);
                } else {
                    // Uncomplete, we 'trash' vector
                    break 'main;
                }
            }
            check!('main, fitted.send_one_vec_i128(vector).await)
        }
    }
}
