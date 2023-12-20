#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

use melodium_core::common::executive::Value;
use melodium_macro::{mel_package, mel_treatment};
use std::collections::VecDeque;

pub mod vec;

pub mod bool;
pub mod byte;
pub mod char;
pub mod f32;
pub mod f64;
pub mod i128;
pub mod i16;
pub mod i32;
pub mod i64;
pub mod i8;
pub mod string;
pub mod u128;
pub mod u16;
pub mod u32;
pub mod u64;
pub mod u8;
pub mod void;

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
    generic G
    input stream Stream<G>
    output start Block<void>
    output end Block<void>
    output first Block<G>
    output last Block<G>
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
    generic F
    input trigger Block<void>
    output emit Block<F>
)]
pub async fn emit(value: F) {
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
    generic Q
    input block Block<Q>
    output stream Stream<Q>
)]
pub async fn stream() {
    if let Ok(val) = block.recv_one().await {
        let _ = stream.send_one(val).await;
    }
}

mel_package!();
