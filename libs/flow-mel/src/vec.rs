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
///     style B fill:#ffff,stroke:#ffff
///     style O fill:#ffff,stroke:#ffff
/// ```
#[mel_treatment(
    generic T
    input vector Stream<Vec<T>>
    output value Stream<T>
)]
pub async fn flatten() {
    'main: while let Ok(vectors) = vector
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        while let Some(vector) = vectors.pop_front().map(|val| match val {
            Value::Vec(vec) => vec,
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
///     style A fill:#ffff,stroke:#ffff
///     style O fill:#ffff,stroke:#ffff
/// ```
#[mel_treatment(
    generic T
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
            match val {
                Value::Vec(vec) => {
                    check!('main, pattern.send_one(vec![(); vec.len()].into()).await)
                }
                _ => panic!("Vec expected"),
            }
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
///     style A fill:#ffff,stroke:#ffff
///     style B fill:#ffff,stroke:#ffff
///     style O fill:#ffff,stroke:#ffff
/// ```
#[mel_treatment(
    generic T
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
            match pattern {
                Value::Vec(pattern) => {
                    let mut vector = Vec::with_capacity(pattern.len());
                    for _ in 0..pattern.len() {
                        if let Ok(val) = value.recv_one().await {
                            vector.push(val);
                        } else {
                            // Uncomplete, we 'trash' vector
                            break 'main;
                        }
                    }
                    check!('main, fitted.send_one(vector.into()).await)
                }
                _ => panic!("Vec expected"),
            }
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
/// style B fill:#ffff,stroke:#ffff
/// style O fill:#ffff,stroke:#ffff
/// ```
#[mel_treatment(
    generic T
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
            match pattern {
                Value::Vec(pattern) => {
                    check!('main, filled.send_one(vec![value.clone(); pattern.len()].into()).await)
                }

                _ => panic!("Vec expected"),
            }
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
///     style V fill:#ffff,stroke:#ffff
///     style P fill:#ffff,stroke:#ffff
/// ```
#[mel_treatment(
    generic T
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
            size.send_many(
                iter.into_iter()
                    .map(|v| match v {
                        Value::Vec(v) => (v.len() as u64),

                        _ => panic!("Vec expected"),
                    })
                    .collect::<VecDeque<_>>()
                    .into()
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
///     style V fill:#ffff,stroke:#ffff
///     style S fill:#ffff,stroke:#ffff
///     style P fill:#ffff,stroke:#ffff
/// ```
#[mel_treatment(
    generic T
    input vector Stream<Vec<T>>
    input size Stream<u64>
    output resized Stream<Vec<T>>
)]
pub async fn resize(default: T) {
    while let Ok(size) = size
        .recv_one()
        .await
        .map(|val| GetData::<u64>::try_data(val).unwrap())
    {
        if let Ok(mut vec) = vector.recv_one().await {
            match vec {
                Value::Vec(vec) => {
                    vec.resize(size as usize, default.clone());
                    check!(resized.send_one(vec.into()).await);
                }
                _ => panic!("Vec expected"),
            }
        } else {
            break;
        }
    }
}
