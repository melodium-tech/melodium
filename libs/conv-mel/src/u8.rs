use melodium_core::*;
use melodium_macro::{check, mel_function, mel_treatment};

/// Turns `u8` stream into `void` one.
#[mel_treatment(
    input value Stream<u8>
    output iter Stream<void>
)]
pub async fn to_void() {
    while let Ok(values) = value.recv_u8().await {
        check!(iter.send_void(vec![(); values.len()]).await)
    }
}

/// Turns `u8` into `Vec<byte>`.
#[mel_function]
pub fn to_byte(value: u8) -> Vec<byte> {
    value.to_be_bytes().to_vec()
}

/// Turns `u8` stream into `byte` one.
///
/// Each `u8` gets converted into `Vec<byte>`, with each vector containing the `bytes`s of the former scalar `u8` it represents.
#[mel_treatment(
    input value Stream<u8>
    output data Stream<Vec<byte>>
)]
pub async fn to_byte() {
    while let Ok(values) = value.recv_u8().await {
        check!(
            data.send_vec_byte(
                values
                    .into_iter()
                    .map(|val| val.to_be_bytes().to_vec())
                    .collect()
            )
            .await
        )
    }
}

/// Turns `u8` into `u16`.
///
/// This conversion is lossless, as any `u8` value can fit into a `u16`.
#[mel_function]
pub fn to_u16(value: u8) -> u16 {
    value as u16
}

/// Turns `u8` stream into `u16` one.
///
/// Each `u8` gets converted into `u16`.
/// This conversion is lossless, as any `u8` value can fit into a `u16`.
#[mel_treatment(
    input value Stream<u8>
    output into Stream<u16>
)]
pub async fn to_u16() {
    while let Ok(values) = value.recv_u8().await {
        check!(
            into.send_u16(values.into_iter().map(|val| val as u16).collect())
                .await
        )
    }
}

/// Turns `u8` into `u32`.
///
/// This conversion is lossless, as any `u8` value can fit into a `u32`.
#[mel_function]
pub fn to_u32(value: u8) -> u32 {
    value as u32
}

/// Turns `u8` stream into `u32` one.
///
/// Each `u8` gets converted into `u32`.
/// This conversion is lossless, as any `u8` value can fit into a `u32`.
#[mel_treatment(
    input value Stream<u8>
    output into Stream<u32>
)]
pub async fn to_u32() {
    while let Ok(values) = value.recv_u8().await {
        check!(
            into.send_u32(values.into_iter().map(|val| val as u32).collect())
                .await
        )
    }
}

/// Turns `u8` into `u64`.
///
/// This conversion is lossless, as any `u8` value can fit into a `u64`.
#[mel_function]
pub fn to_u64(value: u8) -> u64 {
    value as u64
}

/// Turns `u8` stream into `u64` one.
///
/// Each `u8` gets converted into `u64`.
/// This conversion is lossless, as any `u8` value can fit into a `u64`.
#[mel_treatment(
    input value Stream<u8>
    output into Stream<u64>
)]
pub async fn to_u64() {
    while let Ok(values) = value.recv_u8().await {
        check!(
            into.send_u64(values.into_iter().map(|val| val as u64).collect())
                .await
        )
    }
}

/// Turns `u8` into `u128`.
///
/// This conversion is lossless, as any `u8` value can fit into a `u128`.
#[mel_function]
pub fn to_u128(value: u8) -> u128 {
    value as u128
}

/// Turns `u8` stream into `u128` one.
///
/// Each `u8` gets converted into `u128`.
/// This conversion is lossless, as any `u8` value can fit into a `u128`.
#[mel_treatment(
    input value Stream<u8>
    output into Stream<u128>
)]
pub async fn to_u128() {
    while let Ok(values) = value.recv_u8().await {
        check!(
            into.send_u128(values.into_iter().map(|val| val as u128).collect())
                .await
        )
    }
}

/// Turns `u8` into `i16`.
///
/// This conversion is lossless, as any `u8` value can fit into a `i16`.
#[mel_function]
pub fn to_i16(value: u8) -> i16 {
    value as i16
}

/// Turns `u8` stream into `i16` one.
///
/// Each `u8` gets converted into `i16`.
/// This conversion is lossless, as any `u8` value can fit into a `i16`.
#[mel_treatment(
    input value Stream<u8>
    output into Stream<i16>
)]
pub async fn to_i16() {
    while let Ok(values) = value.recv_u8().await {
        check!(
            into.send_i16(values.into_iter().map(|val| val as i16).collect())
                .await
        )
    }
}

/// Turns `u8` into `i32`.
///
/// This conversion is lossless, as any `u8` value can fit into a `i32`.
#[mel_function]
pub fn to_i32(value: u8) -> i32 {
    value as i32
}

/// Turns `u8` stream into `i32` one.
///
/// Each `u8` gets converted into `i32`.
/// This conversion is lossless, as any `u8` value can fit into a `i32`.
#[mel_treatment(
    input value Stream<u8>
    output into Stream<i32>
)]
pub async fn to_i32() {
    while let Ok(values) = value.recv_u8().await {
        check!(
            into.send_i32(values.into_iter().map(|val| val as i32).collect())
                .await
        )
    }
}

/// Turns `u8` into `i64`.
///
/// This conversion is lossless, as any `u8` value can fit into a `i64`.
#[mel_function]
pub fn to_i64(value: u8) -> i64 {
    value as i64
}

/// Turns `u8` stream into `i64` one.
///
/// Each `u8` gets converted into `i64`.
/// This conversion is lossless, as any `u8` value can fit into a `i64`.
#[mel_treatment(
    input value Stream<u8>
    output into Stream<i64>
)]
pub async fn to_i64() {
    while let Ok(values) = value.recv_u8().await {
        check!(
            into.send_i64(values.into_iter().map(|val| val as i64).collect())
                .await
        )
    }
}

/// Turns `u8` into `i128`.
///
/// This conversion is lossless, as any `u8` value can fit into a `i128`.
#[mel_function]
pub fn to_i128(value: u8) -> i128 {
    value as i128
}

/// Turns `u8` stream into `i128` one.
///
/// Each `u8` gets converted into `i128`.
/// This conversion is lossless, as any `u8` value can fit into a `i128`.
#[mel_treatment(
    input value Stream<u8>
    output into Stream<i128>
)]
pub async fn to_i128() {
    while let Ok(values) = value.recv_u8().await {
        check!(
            into.send_i128(values.into_iter().map(|val| val as i128).collect())
                .await
        )
    }
}

/// Turns `u8` into `f32`.
///
/// This conversion is lossless, as any `u8` value can fit into a `f32`.
#[mel_function]
pub fn to_f32(value: u8) -> f32 {
    value as f32
}

/// Turns `u8` stream into `f32` one.
///
/// Each `u8` gets converted into `f32`.
/// This conversion is lossless, as any `u8` value can fit into a `f32`.
#[mel_treatment(
    input value Stream<u8>
    output into Stream<f32>
)]
pub async fn to_f32() {
    while let Ok(values) = value.recv_u8().await {
        check!(
            into.send_f32(values.into_iter().map(|val| val as f32).collect())
                .await
        )
    }
}

/// Turns `u8` into `f64`.
///
/// This conversion is lossless, as any `u8` value can fit into a `f64`.
#[mel_function]
pub fn to_f64(value: u8) -> f64 {
    value as f64
}

/// Turns `u8` stream into `f64` one.
///
/// Each `u8` gets converted into `f64`.
/// This conversion is lossless, as any `u8` value can fit into a `f64`.
#[mel_treatment(
    input value Stream<u8>
    output into Stream<f64>
)]
pub async fn to_f64() {
    while let Ok(values) = value.recv_u8().await {
        check!(
            into.send_f64(values.into_iter().map(|val| val as f64).collect())
                .await
        )
    }
}

/// Turns `u8` into `i8`.
///
/// As this conversion might be lossy (every possible `u8` value cannot fit into `i8`),
/// `truncate` allows value to be truncated to fit into a `i8`, and `or_default` set the
/// value that is assigned when a `u8` is out of range for `i8` and truncation not allowed.
///
/// Truncation happens on the binary level, thus: `10010110` (150 if unsigned, -106 if [signed](https://en.wikipedia.org/wiki/Signed_number_representations)) → `0110` (6).
///
#[mel_function]
pub fn to_i8(value: u8, truncate: bool, or_default: i8) -> i8 {
    if truncate {
        value as i8
    } else {
        use std::convert::TryInto;
        TryInto::<i8>::try_into(value).unwrap_or(or_default)
    }
}

/// Convert stream of `u8` into `i8`.
///
/// As this conversion might be lossy (every possible `u8` value cannot fit into `i8`),
/// `truncate` allows value to be truncated to fit into a `i8`, and `or_default` set the
/// value that is assigned when a `u8` is out of range for `i8` and truncation not allowed.
///
/// Truncation happens on the binary level, thus: `10010110` (150 if unsigned, -106 if [signed](https://en.wikipedia.org/wiki/Signed_number_representations)) → `0110` (6).
///
#[mel_treatment(
    default truncate true
    default or_default 0
    input value Stream<u8>
    output into Stream<i8>
)]
pub async fn to_i8(truncate: bool, or_default: i8) {
    if truncate {
        while let Ok(values) = value.recv_u8().await {
            check!(
                into.send_i8(values.into_iter().map(|val| val as i8).collect())
                    .await
            )
        }
    } else {
        use std::convert::TryInto;
        while let Ok(values) = value.recv_u8().await {
            check!(
                into.send_i8(
                    values
                        .into_iter()
                        .map(|val| TryInto::<i8>::try_into(val).unwrap_or(or_default))
                        .collect()
                )
                .await
            )
        }
    }
}
