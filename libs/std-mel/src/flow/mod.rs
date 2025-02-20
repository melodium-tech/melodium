use futures::{pin_mut, select, FutureExt};
use melodium_core::common::executive::{GetData, Value};
use melodium_macro::{check, mel_treatment};
use std::collections::VecDeque;

pub mod concentrate;
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
///     B["🟥 … 🟨 🟨 🟨 🟨 🟨 🟨 … 🟩"] -->|stream| T
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
            let _ = first.send_one(val.clone()).await;
            last_value = Some(val);
        }
        if let Some(val) = Into::<VecDeque<Value>>::into(values).pop_back() {
            last_value = Some(val);
        }

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

/// Uncheck a blocking value.
///
/// When `value` block stream is closed without receiving anything, `uncheck` is emitted.
///
/// ```mermaid
/// graph LR
///     T("uncheck()")
///     B["〈🟨〉"] -->|value| T
///         
///     T -->|uncheck| S["〈🟦〉"]
///     
///     style B fill:#ffff,stroke:#ffff
///     style S fill:#ffff,stroke:#ffff
/// ```
#[mel_treatment(
    generic T ()
    input value Block<T>
    output uncheck Block<void>
)]
pub async fn uncheck() {
    if let Err(_) = value.recv_one().await {
        let _ = uncheck.send_one(().into()).await;
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
/// The two streams are merged without predictible order.
///
/// ℹ️ Merge continues as long as `a` or `b` continues too, while the other can be ended.
///
/// ```mermaid
/// graph LR
///     T("merge()")
///     A["… 🟦 🟫 …"] -->|a| T
///     B["… 🟧 🟪 🟨 …"] -->|b| T
///     
///
///     T -->|value| V["… 🟦 🟧 🟪 🟫 🟨 …"]
///
///     style V fill:#ffff,stroke:#ffff
///     style A fill:#ffff,stroke:#ffff
///     style B fill:#ffff,stroke:#ffff
/// ```
#[mel_treatment(
    generic T ()
    input a Stream<T>
    input b Stream<T>
    output value Stream<T>
)]
pub async fn merge() {
    let xa = async {
        while let Ok(a) = (&a).recv_many().await {
            check!(value.send_many(a).await);
        }
    }
    .fuse();
    let xb = async {
        while let Ok(b) = (&b).recv_many().await {
            check!(value.send_many(b).await);
        }
    }
    .fuse();

    pin_mut!(xa, xb);

    loop {
        select! {
            () = xa => {},
            () = xb => {},
            complete => break,
        };
    }
}

/// Arrange two streams as one.
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
///     T("arrange()")
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
pub async fn arrange() {
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

/// Filter a block according to `bool` value.
///
/// ℹ️ If `select` is never received nothing is emitted.
///  
/// ```mermaid
/// graph LR
///     T("filterBlock()")
///     V["〈🟦〉"] -->|value| T
///     D["〈🟩〉"] -->|select|T
///     
///     T -->|accepted| A["〈🟦〉"]
///     T -->|rejected| R[" "]
///
///     style V fill:#ffff,stroke:#ffff
///     style D fill:#ffff,stroke:#ffff
///     style A fill:#ffff,stroke:#ffff
///     style R fill:#ffff,stroke:#ffff
/// ```
#[mel_treatment(
    generic T ()
    input value Block<T>
    input select Block<bool>
    output accepted Block<T>
    output rejected Block<T>
)]
pub async fn filterBlock() {
    if let (Ok(value), Ok(select)) = futures::join!(value.recv_one(), select.recv_one()) {
        let select = GetData::<bool>::try_data(select).unwrap();
        if select {
            let _ = accepted.send_one(value).await;
        } else {
            let _ = rejected.send_one(value).await;
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

/// Insert a block into a stream.
///
/// `block` is inserted into `stream` when it comes and everything is streamed to `output`.
///
/// ℹ️ No assumption on block insertion position in stream can be made.
///
/// ```mermaid
/// graph LR
///     T("insert()")
///     A["… 🟦 🟦 🟦 🟦 …"] -->|stream| T
///     B["〈🟧〉"] -->|block| T
///     
///
///     T -->|output| V["… 🟦 🟧 🟦 🟦 🟦 …"]
///
///     style V fill:#ffff,stroke:#ffff
///     style A fill:#ffff,stroke:#ffff
///     style B fill:#ffff,stroke:#ffff
/// ```
#[mel_treatment(
    generic T ()
    input stream Stream<T>
    input block Block<T>
    output output Stream<T>
)]
pub async fn insert() {
    let streaming = async {
        while let Ok(values) = (&stream).recv_many().await {
            check!(output.send_many(values).await);
        }
    }
    .fuse();
    let insert_block = async {
        if let Ok(val) = (&block).recv_one().await {
            let _ = output.send_one(val).await;
        }
    }
    .fuse();

    pin_mut!(streaming, insert_block);

    loop {
        select! {
            () = streaming => {},
            () = insert_block => {},
            complete => break,
        };
    }
}

/// Merge two incoming blocks as a stream.
///
/// Each block is taken when it arrives and send through `stream`.
///
/// ℹ️ No priority on blocks order in stream can be assumed.
///
/// ```mermaid
/// graph LR
///     T("flock()")
///     A["〈🟦〉"] -->|a| T
///     B["〈🟧〉"] -->|b| T
///     
///
///     T -->|stream| V["🟧 🟦"]
///
///     style V fill:#ffff,stroke:#ffff
///     style A fill:#ffff,stroke:#ffff
///     style B fill:#ffff,stroke:#ffff
/// ```
#[mel_treatment(
    generic T ()
    input a Block<T>
    input b Block<T>
    output stream Stream<T>
)]
pub async fn flock() {
    let xa = async {
        if let Ok(a) = (&a).recv_one().await {
            let _ = stream.send_one(a).await;
        }
    }
    .fuse();
    let xb = async {
        if let Ok(b) = (&b).recv_one().await {
            let _ = stream.send_one(b).await;
        }
    }
    .fuse();

    pin_mut!(xa, xb);

    loop {
        select! {
            () = xa => {},
            () = xb => {},
            complete => break,
        };
    }
}

/// Emit one block.
///
/// Take first block coming among `a` or `b` and emit it in `value`, ignoring the remaining one.
///
/// ℹ️ No priority between blocks can be assumed if they are ready at same moment.
///
/// ```mermaid
/// graph LR
///     T("one()")
///     A["…"] -->|a| T
///     B["〈🟧〉"] -->|b| T
///     
///
///     T -->|value| V["〈🟧〉"]
///
///     style V fill:#ffff,stroke:#ffff
///     style A fill:#ffff,stroke:#ffff
///     style B fill:#ffff,stroke:#ffff
/// ```
#[mel_treatment(
    generic T ()
    input a Block<T>
    input b Block<T>
    output value Block<T>
)]
pub async fn one() {
    let xa = async { (&a).recv_one().await.ok() }.fuse();
    let xb = async { (&b).recv_one().await.ok() }.fuse();

    pin_mut!(xa, xb);

    loop {
        let val = select! {
            val = xa => val,
            val = xb => val,
            complete => break,
        };

        if let Some(val) = val {
            let _ = value.send_one(val).await;
            break;
        }
    }
}

/// Never send any value.
///
/// No value is ever sent on `closed` output, which is immediately closed.
///
#[mel_treatment(
    generic T ()
    input trigger Block<void>
    output closed Stream<T>
)]
pub async fn close() {
    // Nothing to do
}

/// Consume stream indefinitely.
///
/// Input `stream` is consumed indefinitely until it becomes closed by previous treatment.
///
#[mel_treatment(
    generic T ()
    input stream Stream<T>
)]
pub async fn consume() {
    while let Ok(_) = stream.recv_many().await {
        // Nothing to do.
    }
}

/// Pass stream under condition.
///
/// If `if` is `true`, pass the stream, else closes it.
///
#[mel_treatment(
    generic T ()
    input stream Stream<T>
    output passed Stream<T>
)]
pub async fn pass(cond: bool) {
    if cond {
        while let Ok(data) = stream.recv_many().await {
            check!(passed.send_many(data).await)
        }
    }
}

/// Pass block under condition.
///
/// If `if` is `true`, pass the block, else nothing.
///
#[mel_treatment(
    generic T ()
    input block Block<T>
    output passed Block<T>
)]
pub async fn passBlock(cond: bool) {
    if cond {
        if let Ok(data) = block.recv_one().await {
            let _ = passed.send_one(data).await;
        }
    }
}

/// Barrier stream under condition.
///
/// Awaits `leverage` to let stream pass if it is `true`, else closes the stream.
///
#[mel_treatment(
    generic T ()
    input leverage Block<bool>
    input stream Stream<T>
    output passed Stream<T>
)]
pub async fn barrier() {
    if let Ok(true) = leverage
        .recv_one()
        .await
        .map(|val| GetData::<bool>::try_data(val).unwrap())
    {
        while let Ok(data) = stream.recv_many().await {
            check!(passed.send_many(data).await)
        }
    }
}

/// Pass stream until cut signal.
///
/// Let stream pass until `cut` signal si received.
///
#[mel_treatment(
    generic T ()
    input cut Block<void>
    input stream Stream<T>
    output passed Stream<T>
)]
pub async fn cut() {
    let cut = async { cut.recv_one().await.is_ok() }.fuse();
    let pass = async {
        while let Ok(values) = stream.recv_many().await {
            check!(passed.send_many(values).await)
        }
    }
    .fuse();

    pin_mut!(cut, pass);

    loop {
        select! {
            () = pass => break,
            do_cut = cut => if do_cut {
                break
            },
            complete => break,
        }
    }
}

/// Release stream once signal is received.
///
/// Awaits `leverage` to let stream pass, else closes the stream.
///
#[mel_treatment(
    generic T ()
    input leverage Block<void>
    input data Stream<T>
    output released Stream<T>
)]
pub async fn release() {
    if let Ok(_) = leverage.recv_one().await {
        while let Ok(data) = data.recv_many().await {
            check!(released.send_many(data).await)
        }
    }
}

/// Release block once signal is received.
///
/// Awaits `leverage` to let block pass, else closes the flow.
///
#[mel_treatment(
    generic T ()
    input leverage Block<void>
    input data Block<T>
    output released Block<T>
)]
pub async fn releaseBlock() {
    if let Ok(_) = leverage.recv_one().await {
        if let Ok(data) = data.recv_one().await {
            let _ = released.send_one(data).await;
        }
    }
}

/// Await blocks.
///
/// Wait for two blocks and send `awaited` once both are received.
///
/// ℹ️ If one block is never received, `awaited` is never emitted.
#[mel_treatment(
    generic T ()
    input a Block<T>
    input b Block<T>
    output awaited Block<void>
)]
pub async fn waitBlock() {
    let (a, b) = futures::join!(async { a.recv_one().await.is_ok() }, async {
        b.recv_one().await.is_ok()
    });

    if a && b {
        let _ = awaited.send_one(().into()).await;
    }
}
