#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

use melodium_core::*;
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
    generic T (PartialEquality)
)]
pub fn equal(a: T, b: T) -> bool {
    a.partial_equality_eq(&b)
}

/// Return whether `a` is different `b`
#[mel_function(
    generic T (PartialEquality)
)]
pub fn not_equal(a: T, b: T) -> bool {
    a.partial_equality_ne(&b)
}

/// Determine whether `a` is equal to `b`
#[mel_treatment(
    generic T (PartialEquality)
    input a Stream<T>
    input b Stream<T>
    output result Stream<bool>
)]
pub async fn equal() {
    while let (Ok(a), Ok(b)) = (a.recv_one().await, b.recv_one().await) {
        check!(result.send_one(a.partial_equality_eq(&b).into()).await)
    }
}

/// Determine whether `a` is different from `b`
#[mel_treatment(
    generic T (PartialEquality)
    input a Stream<T>
    input b Stream<T>
    output result Stream<bool>
)]
pub async fn not_equal() {
    while let (Ok(a), Ok(b)) = (a.recv_one().await, b.recv_one().await) {
        check!(result.send_one(a.partial_equality_ne(&b).into()).await)
    }
}

/// Tells whether `a` is strictly greater than `b`.
#[mel_function(
    generic T (PartialOrder)
)]
pub fn gt(a: T, b: T) -> bool {
    a.partial_order_gt(&b)
}

/// Determine whether `a` is strictly greater than `b`
#[mel_treatment(
    generic T (PartialOrder)
    input a Stream<T>
    input b Stream<T>
    output is Stream<bool>
)]
pub async fn greater_than() {
    while let (Ok(a), Ok(b)) = (a.recv_one().await, b.recv_one().await) {
        check!(is.send_one(a.partial_order_gt(&b).into()).await)
    }
}

/// Tells whether `a` is greater or equal to `b`.
#[mel_function(
    generic T (PartialOrder)
)]
pub fn ge(a: T, b: T) -> bool {
    a.partial_order_ge(&b)
}

/// Determine whether `a` is greater or equal to `b`
#[mel_treatment(
    generic T (PartialOrder)
    input a Stream<T>
    input b Stream<T>
    output is Stream<bool>
)]
pub async fn greater_equal() {
    while let (Ok(a), Ok(b)) = (a.recv_one().await, b.recv_one().await) {
        check!(is.send_one(a.partial_order_ge(&b).into()).await)
    }
}

/// Tells whether `a` is strictly lower than `b`.
#[mel_function(
    generic T (PartialOrder)
)]
pub fn lt(a: T, b: T) -> bool {
    a.partial_order_lt(&b)
}

/// Determine whether `a` is strictly lower than `b`
#[mel_treatment(
    generic T (PartialOrder)
    input a Stream<T>
    input b Stream<T>
    output is Stream<bool>
)]
pub async fn lower_than() {
    while let (Ok(a), Ok(b)) = (a.recv_one().await, b.recv_one().await) {
        check!(is.send_one(a.partial_order_lt(&b).into()).await)
    }
}

/// Tells whether `a` is lower or equal to `b`.
#[mel_function(
    generic T (PartialOrder)
)]
pub fn le(a: T, b: T) -> bool {
    a.partial_order_le(&b)
}

/// Determine whether `a` is lower or equal to `b`
#[mel_treatment(
    generic T (PartialOrder)
    input a Stream<T>
    input b Stream<T>
    output is Stream<bool>
)]
pub async fn lower_equal() {
    while let (Ok(a), Ok(b)) = (a.recv_one().await, b.recv_one().await) {
        check!(is.send_one(a.partial_order_le(&b).into()).await)
    }
}

mel_package!();
