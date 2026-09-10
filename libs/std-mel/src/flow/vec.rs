use melodium_core::*;
use melodium_macro::{check, mel_treatment};

/// Flatten a stream of vector.
///
/// All the input vectors are turned into continuous stream of scalar values, keeping order.
/// ```mermaid
/// graph LR
///     T("flatten()")
///     B["［🟦 🟦］［🟦］［🟦 🟦 🟦］"] -->|vector| T
///     
///     T -->|value| O["🟦 🟦 🟦 🟦 🟦 🟦"]
///
///     style B fill:#ffffff,stroke:#ffffff
///     style O fill:#ffffff,stroke:#ffffff
/// ```
#[mel_treatment(
    generic T ()
    input vector Stream<Vec<T>>
    output value Stream<T>
)]
pub async fn flatten() {
    'main: while let Ok(mut vectors) = vector
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        while let Some(vector) = vectors.pop_front().map(|val| match val {
            Value::Vec(vec) => vec,
            Value::Packed(arr) => arr.into_values(),
            _ => panic!("Vec expected"),
        }) {
            for val in vector {
                check!('main, value.send_one(val).await)
            }
        }
    }
}

/// Gives pattern of a stream of vectors.
///
/// ```mermaid
/// graph LR
///     T("pattern()")
///     A["…［🟨 🟨］［🟨］［🟨 🟨 🟨］"] -->|stream| T
///     
///     T -->|pattern| O["… ［🟦 🟦］［🟦］［🟦 🟦 🟦］"]
///
///     style A fill:#ffffff,stroke:#ffffff
///     style O fill:#ffffff,stroke:#ffffff
/// ```
#[mel_treatment(
    generic T ()
    input stream Stream<Vec<T>>
    output pattern Stream<Vec<void>>
)]
pub async fn pattern() {
    'main: while let Ok(vectors) = stream
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        for val in vectors {
            let len = match val {
                Value::Vec(vec) => vec.len(),
                Value::Packed(arr) => arr.len(),
                _ => panic!("Vec expected"),
            };
            check!('main, pattern.send_one_as(vec![(); len]).await)
        }
    }
}

/// Fit a stream of raw values into stream of vectors using a pattern.
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
///     style A fill:#ffffff,stroke:#ffffff
///     style B fill:#ffffff,stroke:#ffffff
///     style O fill:#ffffff,stroke:#ffffff
/// ```
#[mel_treatment(
    generic T ()
    input value Stream<T>
    input pattern Stream<Vec<void>>
    output fitted Stream<Vec<T>>
)]
pub async fn fit() {
    'main: while let Ok(patterns) = pattern
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        for pattern in patterns {
            let pattern_len = match pattern {
                Value::Vec(pattern) => pattern.len(),
                Value::Packed(arr) => arr.len(),
                _ => panic!("Vec expected"),
            };
            let mut vector = Vec::with_capacity(pattern_len);
            for _ in 0..pattern_len {
                if let Ok(val) = value.recv_one().await {
                    vector.push(val);
                } else {
                    // Uncomplete, we 'trash' vector
                    break 'main;
                }
            }
            check!('main, fitted.send_one_as(vector).await)
        }
    }
}

/// Fill a pattern stream with a `i64` value.
///
/// ```mermaid
/// graph LR
/// T("fill(value=🟧)")
/// B["…［🟦 🟦］［🟦］［🟦 🟦 🟦］…"] -->|pattern| T
///
/// T -->|filled| O["…［🟧 🟧］［🟧］［🟧 🟧 🟧］…"]
///
/// style B fill:#ffffff,stroke:#ffffff
/// style O fill:#ffffff,stroke:#ffffff
/// ```
#[mel_treatment(
    generic T ()
    input pattern Stream<Vec<void>>
    output filled Stream<Vec<T>>
)]
pub async fn fill(value: T) {
    'main: while let Ok(patterns) = pattern
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        for pattern in patterns {
            let len = match pattern {
                Value::Vec(pattern) => pattern.len(),
                Value::Packed(arr) => arr.len(),
                _ => panic!("Vec expected"),
            };
            check!('main, filled.send_one_as(vec![value.clone(); len]).await)
        }
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
///     style V fill:#ffffff,stroke:#ffffff
///     style P fill:#ffffff,stroke:#ffffff
/// ```
#[mel_treatment(
    generic T ()
    input vector Stream<Vec<T>>
    output size Stream<u64>
)]
pub async fn size() {
    while let Ok(iter) = vector
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            size.send_many_as(
                iter.into_iter()
                    .map(|v| match v {
                        Value::Vec(v) => v.len() as u64,
                        Value::Packed(arr) => arr.len() as u64,
                        _ => panic!("Vec expected"),
                    })
                    .collect::<Vec<_>>()
            )
            .await
        );
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
///     style V fill:#ffffff,stroke:#ffffff
///     style S fill:#ffffff,stroke:#ffffff
///     style P fill:#ffffff,stroke:#ffffff
/// ```
#[mel_treatment(
    generic T ()
    input vector Stream<Vec<T>>
    input size Stream<u64>
    output resized Stream<Vec<T>>
)]
pub async fn resize(default: T) {
    while let Ok(size) = size.recv_one_as::<u64>().await {
        if let Ok(vec) = vector.recv_one().await {
            let mut vec = match vec {
                Value::Vec(vec) => vec,
                Value::Packed(arr) => arr.into_values(),
                _ => panic!("Vec expected"),
            };
            vec.resize(size as usize, default.clone());
            check!(resized.send_one_as(vec).await);
        } else {
            break;
        }
    }
}
