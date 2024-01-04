#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

use melodium_macro::{check, mel_function, mel_package, mel_treatment};

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

/// Return whether `a` is equal to `b`
#[mel_function(
    generic T
)]
pub fn equal(a: T, b: T) -> bool {
    a == b
}

/// Return whether `a` is different `b`
#[mel_function(
    generic T
)]
pub fn not_equal(a: T, b: T) -> bool {
    a != b
}

/// Determine whether `a` is equal to `b`
#[mel_treatment(
    generic T
    input a Stream<T>
    input b Stream<T>
    output result Stream<bool>
)]
pub async fn equal() {
    while let (Ok(a), Ok(b)) = (a.recv_one().await, b.recv_one().await) {
        check!(result.send_one((a == b).into()).await)
    }
}

/// Determine whether `a` is different from `b`
#[mel_treatment(
    generic T
    input a Stream<T>
    input b Stream<T>
    output result Stream<bool>
)]
pub async fn not_equal() {
    while let (Ok(a), Ok(b)) = (a.recv_one().await, b.recv_one().await) {
        check!(result.send_one((a != b).into()).await)
    }
}

mel_package!();
