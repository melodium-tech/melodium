#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

use melodium_core::*;
use melodium_macro::{check, mel_function, mel_package, mel_treatment};

pub mod bin;
pub mod float;
pub mod num;

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

/// Gives the minimum value between `a` and `b`.
#[mel_function(
    generic T (Order)
)]
pub fn min(a: T, b: T) -> T {
    a.order_min(&b)
}

/// Stream the minimum value between `a` and `b`.
#[mel_treatment(
    generic T (Order)
    input a Stream<T>
    input b Stream<T>
    output min Stream<T>
)]
pub async fn min() {
    while let (Ok(a), Ok(b)) = (a.recv_one().await, b.recv_one().await) {
        check!(min.send_one(a.order_min(&b)).await)
    }
}

/// Gives the maximum value between `a` and `b`.
#[mel_function(
    generic T (Order)
)]
pub fn max(a: T, b: T) -> T {
    a.order_max(&b)
}

/// Stream the maximum value between `a` and `b`.
#[mel_treatment(
    generic T (Order)
    input a Stream<T>
    input b Stream<T>
    output max Stream<T>
)]
pub async fn max() {
    while let (Ok(a), Ok(b)) = (a.recv_one().await, b.recv_one().await) {
        check!(max.send_one(a.order_max(&b)).await)
    }
}

/// Restrain value between `min` and `max`.
///
/// Value is returned if it is in the [`min`,`max`] interval.
/// If value is smaller `min` is returned, or if it greater `max` is returned.
#[mel_function(
    generic T (Order)
)]
pub fn clamp(value: T, min: T, max: T) -> T {
    value.order_clamp(&min, &max)
}

/// Stream value restrained between `min` and `max`.
///
/// Value is streamed if it is in the [`min`,`max`] interval.
/// If value is smaller `min` is sent, or if it greater `max` is sent.
#[mel_treatment(
    generic T (Order)
    input value Stream<T>
    output clamped Stream<T>
)]
pub async fn clamp(min: T, max: T) {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            clamped
                .send_many(TransmissionValue::Other(
                    values
                        .into_iter()
                        .map(|val| val.order_clamp(&min, &max))
                        .collect()
                ))
                .await
        )
    }
}

mel_package!();
