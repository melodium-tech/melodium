use melodium_core::common::executive::{GetData, Value};
use melodium_macro::{check, mel_treatment};
use std::collections::VecDeque;

pub mod vec;

/// Chain two streams.
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
    generic T ()
    input first Stream<T>
    input second Stream<T>
    output chained Stream<T>
)]
pub async fn chain() {
    while let Ok(values) = first.recv_many().await {
        check!(chained.send_many(values).await)
    }

    while let Ok(values) = second.recv_many().await {
        check!(chained.send_many(values).await)
    }
}

/// Trigger on a stream start and end.
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
    generic T ()
    input stream Stream<T>
    output start Block<void>
    output end Block<void>
    output first Block<T>
    output last Block<T>
)]
pub async fn trigger() {
    let mut last_value = None;

    if let Ok(mut values) = stream.recv_many().await {
        let _ = start.send_one(().into()).await;
        if let Some(val) = values.pop_front() {
            let _ = first.send_one(val).await;
        }
        last_value = Into::<VecDeque<Value>>::into(values).pop_back();
        let _ = futures::join!(start.close(), first.close());
    }

    while let Ok(values) = stream.recv_many().await {
        last_value = Into::<VecDeque<Value>>::into(values).pop_back();
    }

    let _ = end.send_one(().into()).await;
    if let Some(val) = last_value {
        let _ = last.send_one(val).await;
    }

    // We don't close `end` and `last` explicitly here,
    // because it would be redundant with boilerplate
    // implementation of treatments.
}

/// Check a blocking value.
///
/// When `value` block is received, `check` is emitted.
///
/// ```mermaid
/// graph LR
///     T("check()")
///     B["〈🟨〉"] -->|value| T
///         
///     T -->|check| S["〈🟦〉"]
///     
///     style B fill:#ffff,stroke:#ffff
///     style S fill:#ffff,stroke:#ffff
/// ```
#[mel_treatment(
    generic T ()
    input value Block<T>
    output check Block<void>
)]
pub async fn check() {
    if let Ok(_) = value.recv_one().await {
        let _ = check.send_one(().into()).await;
    }
}

/// Emit a blocking value.
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
    generic T ()
    input trigger Block<void>
    output emit Block<T>
)]
pub async fn emit(value: T) {
    if let Ok(_) = trigger.recv_one().await {
        let _ = emit.send_one(value).await;
    }
}

/// Stream a blocking value.
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
    generic T ()
    input block Block<T>
    output stream Stream<T>
)]
pub async fn stream() {
    if let Ok(val) = block.recv_one().await {
        let _ = stream.send_one(val).await;
    }
}

/// Merge two streams.
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
    generic T ()
    input a Stream<T>
    input b Stream<T>
    input select Stream<bool>
    output value Stream<T>
)]
pub async fn merge() {
    while let Ok(select) = select
        .recv_one()
        .await
        .map(|val| GetData::<bool>::try_data(val).unwrap())
    {
        let val;
        if select {
            if let Ok(v) = a.recv_one().await {
                val = v;
            } else {
                break;
            }
        } else {
            if let Ok(v) = b.recv_one().await {
                val = v;
            } else {
                break;
            }
        }

        check!(value.send_one(val).await)
    }
}

/// Fill a pattern stream with a `value.
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
    generic T ()
    input pattern Stream<void>
    output filled Stream<T>
)]
pub async fn fill(value: T) {
    while let Ok(pat) = pattern.recv_many().await {
        let mut transmission = melodium_core::TransmissionValue::new(value.clone());
        for _ in 1..pat.len() {
            transmission.push(value.clone());
        }
        check!(filled.send_many(transmission).await)
    }
}

/// Filter a stream according to `bool` stream.
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
    generic T ()
    input value Stream<T>
    input select Stream<bool>
    output accepted Stream<T>
    output rejected Stream<T>
)]
pub async fn filter() {
    let mut accepted_op = true;
    let mut rejected_op = true;

    while let (Ok(value), Ok(select)) = futures::join!(value.recv_one(), select.recv_one()) {
        let select = GetData::<bool>::try_data(select).unwrap();
        if select {
            if let Err(_) = accepted.send_one(value).await {
                // If we cannot send anymore on accepted, we note it,
                // and check if rejected is still valid, else just terminate.
                accepted_op = false;
                if !rejected_op {
                    break;
                }
            }
        } else {
            if let Err(_) = rejected.send_one(value).await {
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

/// Fit a stream into a pattern.
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
    generic T ()
    input value Stream<T>
    input pattern Stream<void>
    output fitted Stream<T>
)]
pub async fn fit() {
    'main: while let Ok(pattern) = pattern
        .recv_many()
        .await
        .map(|values| TryInto::<Vec<()>>::try_into(values).unwrap())
    {
        for _ in pattern {
            if let Ok(val) = value.recv_one().await {
                check!('main, fitted.send_one(val).await)
            } else {
                break 'main;
            }
        }
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
    generic T ()
    input stream Stream<T>
    output count Stream<u128>
)]
pub async fn count() {
    let mut i: u128 = 0;
    while let Ok(iter) = stream.recv_many().await {
        let next_i = i + iter.len() as u128;
        check!(
            count
                .send_many((i..next_i).collect::<VecDeque<_>>().into())
                .await
        );
        i = next_i;
    }
}

/// Generate a stream with a given length.
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
    generic T ()
    input length Block<u128>
    output stream Stream<T>
)]
pub async fn generate(data: T) {
    if let Ok(length) = length
        .recv_one()
        .await
        .map(|val| GetData::<u128>::try_data(val).unwrap())
    {
        const CHUNK: u128 = 2u128.pow(20);
        let mut total = 0u128;
        while total < length {
            let chunk = u128::min(CHUNK, length - total) as usize;
            let mut transmission = melodium_core::TransmissionValue::new(data.clone());
            for _ in 1..chunk {
                transmission.push(data.clone());
            }
            check!(stream.send_many(transmission).await);
            total += chunk as u128;
        }
    }
}

/// Generate a stream indefinitely.
///
/// This generates a continuous stream, until stream consumers closes it.
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
    generic T ()
    input trigger Block<void>
    output stream Stream<T>
)]
pub async fn generate_indefinitely(data: T) {
    if let Ok(_) = trigger.recv_one().await {
        const CHUNK: usize = 2usize.pow(20);
        loop {
            let mut transmission = melodium_core::TransmissionValue::new(data.clone());
            for _ in 1..CHUNK {
                transmission.push(data.clone());
            }
            check!(stream.send_many(transmission).await);
        }
    }
}
