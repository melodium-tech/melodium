use melodium_macro::{check, mel_function, mel_treatment};
use melodium_core::*;

/// Turns `i8` stream into `void` one.
#[mel_treatment(
    input value Stream<i8>
    output iter Stream<void>
)]
pub async fn to_void() {
    while let Ok(values) = value.recv_i8().await {
        check!(iter.send_void(vec![(); values.len()]).await)
    }
}


/// Turns `i8` stream into `byte` one.
/// 
/// Each `i8` gets converted into `Vec<byte>`, with each vector containing the `bytes`s of the former scalar `i8` it represents.
#[mel_treatment(
    input value Stream<i8>
    output data Stream<Vec<byte>>
)]
pub async fn to_byte() {
    while let Ok(values) = value.recv_i8().await {
        check!(data.send_vec_byte(values.into_iter().map(|val| val.to_be_bytes().to_vec()).collect()).await)
    }
}

/// Turns `i8` stream into `i16` one.
/// 
/// Each `i8` gets converted into `i16`.
/// This conversion is lossless, as any `i8` value can fit into a `i16`.
#[mel_treatment(
    input value Stream<i8>
    output into Stream<i16>
)]
pub async fn to_i16() {
    while let Ok(values) = value.recv_i8().await {
        check!(into.send_i16(values.into_iter().map(|val| val as i16).collect()).await)
    }
}

/// Turns `i8` stream into `i32` one.
/// 
/// Each `i8` gets converted into `i32`.
/// This conversion is lossless, as any `i8` value can fit into a `i32`.
#[mel_treatment(
    input value Stream<i8>
    output into Stream<i32>
)]
pub async fn to_i32() {
    while let Ok(values) = value.recv_i8().await {
        check!(into.send_i32(values.into_iter().map(|val| val as i32).collect()).await)
    }
}

/// Turns `i8` stream into `i64` one.
/// 
/// Each `i8` gets converted into `i64`.
/// This conversion is lossless, as any `i8` value can fit into a `i64`.
#[mel_treatment(
    input value Stream<i8>
    output into Stream<i64>
)]
pub async fn to_i64() {
    while let Ok(values) = value.recv_i8().await {
        check!(into.send_i64(values.into_iter().map(|val| val as i64).collect()).await)
    }
}

/// Turns `i8` stream into `i128` one.
/// 
/// Each `i8` gets converted into `i128`.
/// This conversion is lossless, as any `i8` value can fit into a `i128`.
#[mel_treatment(
    input value Stream<i8>
    output into Stream<i128>
)]
pub async fn to_i128() {
    while let Ok(values) = value.recv_i8().await {
        check!(into.send_i128(values.into_iter().map(|val| val as i128).collect()).await)
    }
}

/// Turns `i8` stream into `f32` one.
/// 
/// Each `i8` gets converted into `f32`.
/// This conversion is lossless, as any `i8` value can fit into a `f32`.
#[mel_treatment(
    input value Stream<i8>
    output into Stream<f32>
)]
pub async fn to_f32() {
    while let Ok(values) = value.recv_i8().await {
        check!(into.send_f32(values.into_iter().map(|val| val as f32).collect()).await)
    }
}

/// Turns `i8` stream into `f64` one.
/// 
/// Each `i8` gets converted into `f64`.
/// This conversion is lossless, as any `i8` value can fit into a `f64`.
#[mel_treatment(
    input value Stream<i8>
    output into Stream<f64>
)]
pub async fn to_f64() {
    while let Ok(values) = value.recv_i8().await {
        check!(into.send_f64(values.into_iter().map(|val| val as f64).collect()).await)
    }
}

/// Convert stream of `i8` into `u8`.
/// 
/// As this conversion might be lossy (every possible `i8` value cannot fit into `u8`),
/// `truncate` allows value to be truncated to fit into a `u8`, and `or_default` set the
/// value that is assigned when a `i8` is out of range for `u8` and truncation not allowed.
/// 
/// Truncation happens on the binary level, thus: `10010110` (150 if unsigned, -106 if [signed](https://en.wikipedia.org/wiki/Signed_number_representations)) → `0110` (6).
/// 
#[mel_treatment(
    default truncate true
    default or_default 0
    input value Stream<i8>
    output into Stream<u8>
)]
pub async fn to_u8(truncate: bool, or_default: u8) {
    
    if truncate {
        while let Ok(values) = value.recv_i8().await {
            check!(into.send_u8(values.into_iter().map(|val| val as u8).collect()).await)
        }
    }
    else {
        use std::convert::TryInto;
        while let Ok(values) = value.recv_i8().await {
            check!(into.send_u8(values.into_iter().map(|val|
                TryInto::<u8>::try_into(val).unwrap_or(or_default)
            ).collect()).await)
        }
    }
}

/// Convert stream of `i8` into `u16`.
/// 
/// As this conversion might be lossy (every possible `i8` value cannot fit into `u16`),
/// `truncate` allows value to be truncated to fit into a `u16`, and `or_default` set the
/// value that is assigned when a `i8` is out of range for `u16` and truncation not allowed.
/// 
/// Truncation happens on the binary level, thus: `10010110` (150 if unsigned, -106 if [signed](https://en.wikipedia.org/wiki/Signed_number_representations)) → `0110` (6).
/// 
#[mel_treatment(
    default truncate true
    default or_default 0
    input value Stream<i8>
    output into Stream<u16>
)]
pub async fn to_u16(truncate: bool, or_default: u16) {
    
    if truncate {
        while let Ok(values) = value.recv_i8().await {
            check!(into.send_u16(values.into_iter().map(|val| val as u16).collect()).await)
        }
    }
    else {
        use std::convert::TryInto;
        while let Ok(values) = value.recv_i8().await {
            check!(into.send_u16(values.into_iter().map(|val|
                TryInto::<u16>::try_into(val).unwrap_or(or_default)
            ).collect()).await)
        }
    }
}

/// Convert stream of `i8` into `u32`.
/// 
/// As this conversion might be lossy (every possible `i8` value cannot fit into `u32`),
/// `truncate` allows value to be truncated to fit into a `u32`, and `or_default` set the
/// value that is assigned when a `i8` is out of range for `u32` and truncation not allowed.
/// 
/// Truncation happens on the binary level, thus: `10010110` (150 if unsigned, -106 if [signed](https://en.wikipedia.org/wiki/Signed_number_representations)) → `0110` (6).
/// 
#[mel_treatment(
    default truncate true
    default or_default 0
    input value Stream<i8>
    output into Stream<u32>
)]
pub async fn to_u32(truncate: bool, or_default: u32) {
    
    if truncate {
        while let Ok(values) = value.recv_i8().await {
            check!(into.send_u32(values.into_iter().map(|val| val as u32).collect()).await)
        }
    }
    else {
        use std::convert::TryInto;
        while let Ok(values) = value.recv_i8().await {
            check!(into.send_u32(values.into_iter().map(|val|
                TryInto::<u32>::try_into(val).unwrap_or(or_default)
            ).collect()).await)
        }
    }
}

/// Convert stream of `i8` into `u64`.
/// 
/// As this conversion might be lossy (every possible `i8` value cannot fit into `u64`),
/// `truncate` allows value to be truncated to fit into a `u64`, and `or_default` set the
/// value that is assigned when a `i8` is out of range for `u64` and truncation not allowed.
/// 
/// Truncation happens on the binary level, thus: `10010110` (150 if unsigned, -106 if [signed](https://en.wikipedia.org/wiki/Signed_number_representations)) → `0110` (6).
/// 
#[mel_treatment(
    default truncate true
    default or_default 0
    input value Stream<i8>
    output into Stream<u64>
)]
pub async fn to_u64(truncate: bool, or_default: u64) {
    
    if truncate {
        while let Ok(values) = value.recv_i8().await {
            check!(into.send_u64(values.into_iter().map(|val| val as u64).collect()).await)
        }
    }
    else {
        use std::convert::TryInto;
        while let Ok(values) = value.recv_i8().await {
            check!(into.send_u64(values.into_iter().map(|val|
                TryInto::<u64>::try_into(val).unwrap_or(or_default)
            ).collect()).await)
        }
    }
}

/// Convert stream of `i8` into `u128`.
/// 
/// As this conversion might be lossy (every possible `i8` value cannot fit into `u128`),
/// `truncate` allows value to be truncated to fit into a `u128`, and `or_default` set the
/// value that is assigned when a `i8` is out of range for `u128` and truncation not allowed.
/// 
/// Truncation happens on the binary level, thus: `10010110` (150 if unsigned, -106 if [signed](https://en.wikipedia.org/wiki/Signed_number_representations)) → `0110` (6).
/// 
#[mel_treatment(
    default truncate true
    default or_default 0
    input value Stream<i8>
    output into Stream<u128>
)]
pub async fn to_u128(truncate: bool, or_default: u128) {
    
    if truncate {
        while let Ok(values) = value.recv_i8().await {
            check!(into.send_u128(values.into_iter().map(|val| val as u128).collect()).await)
        }
    }
    else {
        use std::convert::TryInto;
        while let Ok(values) = value.recv_i8().await {
            check!(into.send_u128(values.into_iter().map(|val|
                TryInto::<u128>::try_into(val).unwrap_or(or_default)
            ).collect()).await)
        }
    }
}
