use melodium_core::*;
use melodium_macro::{check, mel_function, mel_treatment};

/// Turns `i64` stream into `void` one.
#[mel_treatment(
    input value Stream<i64>
    output iter Stream<void>
)]
pub async fn to_void() {
    while let Ok(values) = value.recv_many().await {
        check!(iter.send_many(vec![(); values.len()].into()).await)
    }
}

/// Turns `i64` into `Vec<byte>`.
#[mel_function]
pub fn to_byte(value: i64) -> Vec<byte> {
    value.to_be_bytes().to_vec()
}

/// Turns `i64` stream into `byte` one.
///
/// Each `i64` gets converted into `Vec<byte>`, with each vector containing the `bytes`s of the former scalar `i64` it represents.
#[mel_treatment(
    input value Stream<i64>
    output data Stream<Vec<byte>>
)]
pub async fn to_byte() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| TryInto::<Vec<i64>>::try_into(values).unwrap())
    {
        check!(
            data.send_many(TransmissionValue::Other(
                values
                    .into_iter()
                    .map(|val| Value::Vec(
                        val.to_be_bytes().iter().map(|v| Value::Byte(*v)).collect()
                    ))
                    .collect()
            ))
            .await
        )
    }
}

/// Turns `i64` into `i128`.
///
/// This conversion is lossless, as any `i64` value can fit into a `i128`.
#[mel_function]
pub fn to_i128(value: i64) -> i128 {
    value as i128
}

/// Turns `i64` stream into `i128` one.
///
/// Each `i64` gets converted into `i128`.
/// This conversion is lossless, as any `i64` value can fit into a `i128`.
#[mel_treatment(
    input value Stream<i64>
    output into Stream<i128>
)]
pub async fn to_i128() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| TryInto::<Vec<i64>>::try_into(values).unwrap())
    {
        check!(
            into.send_many(
                values
                    .into_iter()
                    .map(|val| val as i128)
                    .collect::<VecDeque<_>>()
                    .into()
            )
            .await
        )
    }
}

/// Turns `i64` into `f32`.
///
/// This conversion is lossless, as any `i64` value can fit into a `f32`.
#[mel_function]
pub fn to_f32(value: i64) -> f32 {
    value as f32
}

/// Turns `i64` stream into `f32` one.
///
/// Each `i64` gets converted into `f32`.
/// This conversion is lossless, as any `i64` value can fit into a `f32`.
#[mel_treatment(
    input value Stream<i64>
    output into Stream<f32>
)]
pub async fn to_f32() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| TryInto::<Vec<i64>>::try_into(values).unwrap())
    {
        check!(
            into.send_many(
                values
                    .into_iter()
                    .map(|val| val as f32)
                    .collect::<VecDeque<_>>()
                    .into()
            )
            .await
        )
    }
}

/// Turns `i64` into `f64`.
///
/// This conversion is lossless, as any `i64` value can fit into a `f64`.
#[mel_function]
pub fn to_f64(value: i64) -> f64 {
    value as f64
}

/// Turns `i64` stream into `f64` one.
///
/// Each `i64` gets converted into `f64`.
/// This conversion is lossless, as any `i64` value can fit into a `f64`.
#[mel_treatment(
    input value Stream<i64>
    output into Stream<f64>
)]
pub async fn to_f64() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| TryInto::<Vec<i64>>::try_into(values).unwrap())
    {
        check!(
            into.send_many(
                values
                    .into_iter()
                    .map(|val| val as f64)
                    .collect::<VecDeque<_>>()
                    .into()
            )
            .await
        )
    }
}

/// Turns `i64` into `u8`.
///
/// As this conversion might be lossy (every possible `i64` value cannot fit into `u8`),
/// `truncate` allows value to be truncated to fit into a `u8`, and `or_default` set the
/// value that is assigned when a `i64` is out of range for `u8` and truncation not allowed.
///
/// Truncation happens on the binary level, thus: `10010110` (150 if unsigned, -106 if [signed](https://en.wikipedia.org/wiki/Signed_number_representations)) → `0110` (6).
///
#[mel_function]
pub fn to_u8(value: i64, truncate: bool, or_default: u8) -> u8 {
    if truncate {
        value as u8
    } else {
        use std::convert::TryInto;
        TryInto::<u8>::try_into(value).unwrap_or(or_default)
    }
}

/// Convert stream of `i64` into `u8`.
///
/// As this conversion might be lossy (every possible `i64` value cannot fit into `u8`),
/// `truncate` allows value to be truncated to fit into a `u8`, and `or_default` set the
/// value that is assigned when a `i64` is out of range for `u8` and truncation not allowed.
///
/// Truncation happens on the binary level, thus: `10010110` (150 if unsigned, -106 if [signed](https://en.wikipedia.org/wiki/Signed_number_representations)) → `0110` (6).
///
#[mel_treatment(
    default truncate true
    default or_default 0
    input value Stream<i64>
    output into Stream<u8>
)]
pub async fn to_u8(truncate: bool, or_default: u8) {
    if truncate {
        while let Ok(values) = value
            .recv_many()
            .await
            .map(|values| TryInto::<Vec<i64>>::try_into(values).unwrap())
        {
            check!(
                into.send_many(
                    values
                        .into_iter()
                        .map(|val| val as u8)
                        .collect::<VecDeque<_>>()
                        .into()
                )
                .await
            )
        }
    } else {
        use std::convert::TryInto;
        while let Ok(values) = value
            .recv_many()
            .await
            .map(|values| TryInto::<Vec<i64>>::try_into(values).unwrap())
        {
            check!(
                into.send_many(
                    values
                        .into_iter()
                        .map(|val| TryInto::<u8>::try_into(val).unwrap_or(or_default))
                        .collect::<VecDeque<_>>()
                        .into()
                )
                .await
            )
        }
    }
}

/// Turns `i64` into `u16`.
///
/// As this conversion might be lossy (every possible `i64` value cannot fit into `u16`),
/// `truncate` allows value to be truncated to fit into a `u16`, and `or_default` set the
/// value that is assigned when a `i64` is out of range for `u16` and truncation not allowed.
///
/// Truncation happens on the binary level, thus: `10010110` (150 if unsigned, -106 if [signed](https://en.wikipedia.org/wiki/Signed_number_representations)) → `0110` (6).
///
#[mel_function]
pub fn to_u16(value: i64, truncate: bool, or_default: u16) -> u16 {
    if truncate {
        value as u16
    } else {
        use std::convert::TryInto;
        TryInto::<u16>::try_into(value).unwrap_or(or_default)
    }
}

/// Convert stream of `i64` into `u16`.
///
/// As this conversion might be lossy (every possible `i64` value cannot fit into `u16`),
/// `truncate` allows value to be truncated to fit into a `u16`, and `or_default` set the
/// value that is assigned when a `i64` is out of range for `u16` and truncation not allowed.
///
/// Truncation happens on the binary level, thus: `10010110` (150 if unsigned, -106 if [signed](https://en.wikipedia.org/wiki/Signed_number_representations)) → `0110` (6).
///
#[mel_treatment(
    default truncate true
    default or_default 0
    input value Stream<i64>
    output into Stream<u16>
)]
pub async fn to_u16(truncate: bool, or_default: u16) {
    if truncate {
        while let Ok(values) = value
            .recv_many()
            .await
            .map(|values| TryInto::<Vec<i64>>::try_into(values).unwrap())
        {
            check!(
                into.send_many(
                    values
                        .into_iter()
                        .map(|val| val as u16)
                        .collect::<VecDeque<_>>()
                        .into()
                )
                .await
            )
        }
    } else {
        use std::convert::TryInto;
        while let Ok(values) = value
            .recv_many()
            .await
            .map(|values| TryInto::<Vec<i64>>::try_into(values).unwrap())
        {
            check!(
                into.send_many(
                    values
                        .into_iter()
                        .map(|val| TryInto::<u16>::try_into(val).unwrap_or(or_default))
                        .collect::<VecDeque<_>>()
                        .into()
                )
                .await
            )
        }
    }
}

/// Turns `i64` into `u32`.
///
/// As this conversion might be lossy (every possible `i64` value cannot fit into `u32`),
/// `truncate` allows value to be truncated to fit into a `u32`, and `or_default` set the
/// value that is assigned when a `i64` is out of range for `u32` and truncation not allowed.
///
/// Truncation happens on the binary level, thus: `10010110` (150 if unsigned, -106 if [signed](https://en.wikipedia.org/wiki/Signed_number_representations)) → `0110` (6).
///
#[mel_function]
pub fn to_u32(value: i64, truncate: bool, or_default: u32) -> u32 {
    if truncate {
        value as u32
    } else {
        use std::convert::TryInto;
        TryInto::<u32>::try_into(value).unwrap_or(or_default)
    }
}

/// Convert stream of `i64` into `u32`.
///
/// As this conversion might be lossy (every possible `i64` value cannot fit into `u32`),
/// `truncate` allows value to be truncated to fit into a `u32`, and `or_default` set the
/// value that is assigned when a `i64` is out of range for `u32` and truncation not allowed.
///
/// Truncation happens on the binary level, thus: `10010110` (150 if unsigned, -106 if [signed](https://en.wikipedia.org/wiki/Signed_number_representations)) → `0110` (6).
///
#[mel_treatment(
    default truncate true
    default or_default 0
    input value Stream<i64>
    output into Stream<u32>
)]
pub async fn to_u32(truncate: bool, or_default: u32) {
    if truncate {
        while let Ok(values) = value
            .recv_many()
            .await
            .map(|values| TryInto::<Vec<i64>>::try_into(values).unwrap())
        {
            check!(
                into.send_many(
                    values
                        .into_iter()
                        .map(|val| val as u32)
                        .collect::<VecDeque<_>>()
                        .into()
                )
                .await
            )
        }
    } else {
        use std::convert::TryInto;
        while let Ok(values) = value
            .recv_many()
            .await
            .map(|values| TryInto::<Vec<i64>>::try_into(values).unwrap())
        {
            check!(
                into.send_many(
                    values
                        .into_iter()
                        .map(|val| TryInto::<u32>::try_into(val).unwrap_or(or_default))
                        .collect::<VecDeque<_>>()
                        .into()
                )
                .await
            )
        }
    }
}

/// Turns `i64` into `u64`.
///
/// As this conversion might be lossy (every possible `i64` value cannot fit into `u64`),
/// `truncate` allows value to be truncated to fit into a `u64`, and `or_default` set the
/// value that is assigned when a `i64` is out of range for `u64` and truncation not allowed.
///
/// Truncation happens on the binary level, thus: `10010110` (150 if unsigned, -106 if [signed](https://en.wikipedia.org/wiki/Signed_number_representations)) → `0110` (6).
///
#[mel_function]
pub fn to_u64(value: i64, truncate: bool, or_default: u64) -> u64 {
    if truncate {
        value as u64
    } else {
        use std::convert::TryInto;
        TryInto::<u64>::try_into(value).unwrap_or(or_default)
    }
}

/// Convert stream of `i64` into `u64`.
///
/// As this conversion might be lossy (every possible `i64` value cannot fit into `u64`),
/// `truncate` allows value to be truncated to fit into a `u64`, and `or_default` set the
/// value that is assigned when a `i64` is out of range for `u64` and truncation not allowed.
///
/// Truncation happens on the binary level, thus: `10010110` (150 if unsigned, -106 if [signed](https://en.wikipedia.org/wiki/Signed_number_representations)) → `0110` (6).
///
#[mel_treatment(
    default truncate true
    default or_default 0
    input value Stream<i64>
    output into Stream<u64>
)]
pub async fn to_u64(truncate: bool, or_default: u64) {
    if truncate {
        while let Ok(values) = value
            .recv_many()
            .await
            .map(|values| TryInto::<Vec<i64>>::try_into(values).unwrap())
        {
            check!(
                into.send_many(
                    values
                        .into_iter()
                        .map(|val| val as u64)
                        .collect::<VecDeque<_>>()
                        .into()
                )
                .await
            )
        }
    } else {
        use std::convert::TryInto;
        while let Ok(values) = value
            .recv_many()
            .await
            .map(|values| TryInto::<Vec<i64>>::try_into(values).unwrap())
        {
            check!(
                into.send_many(
                    values
                        .into_iter()
                        .map(|val| TryInto::<u64>::try_into(val).unwrap_or(or_default))
                        .collect::<VecDeque<_>>()
                        .into()
                )
                .await
            )
        }
    }
}

/// Turns `i64` into `u128`.
///
/// As this conversion might be lossy (every possible `i64` value cannot fit into `u128`),
/// `truncate` allows value to be truncated to fit into a `u128`, and `or_default` set the
/// value that is assigned when a `i64` is out of range for `u128` and truncation not allowed.
///
/// Truncation happens on the binary level, thus: `10010110` (150 if unsigned, -106 if [signed](https://en.wikipedia.org/wiki/Signed_number_representations)) → `0110` (6).
///
#[mel_function]
pub fn to_u128(value: i64, truncate: bool, or_default: u128) -> u128 {
    if truncate {
        value as u128
    } else {
        use std::convert::TryInto;
        TryInto::<u128>::try_into(value).unwrap_or(or_default)
    }
}

/// Convert stream of `i64` into `u128`.
///
/// As this conversion might be lossy (every possible `i64` value cannot fit into `u128`),
/// `truncate` allows value to be truncated to fit into a `u128`, and `or_default` set the
/// value that is assigned when a `i64` is out of range for `u128` and truncation not allowed.
///
/// Truncation happens on the binary level, thus: `10010110` (150 if unsigned, -106 if [signed](https://en.wikipedia.org/wiki/Signed_number_representations)) → `0110` (6).
///
#[mel_treatment(
    default truncate true
    default or_default 0
    input value Stream<i64>
    output into Stream<u128>
)]
pub async fn to_u128(truncate: bool, or_default: u128) {
    if truncate {
        while let Ok(values) = value
            .recv_many()
            .await
            .map(|values| TryInto::<Vec<i64>>::try_into(values).unwrap())
        {
            check!(
                into.send_many(
                    values
                        .into_iter()
                        .map(|val| val as u128)
                        .collect::<VecDeque<_>>()
                        .into()
                )
                .await
            )
        }
    } else {
        use std::convert::TryInto;
        while let Ok(values) = value
            .recv_many()
            .await
            .map(|values| TryInto::<Vec<i64>>::try_into(values).unwrap())
        {
            check!(
                into.send_many(
                    values
                        .into_iter()
                        .map(|val| TryInto::<u128>::try_into(val).unwrap_or(or_default))
                        .collect::<VecDeque<_>>()
                        .into()
                )
                .await
            )
        }
    }
}

/// Turns `i64` into `i8`.
///
/// As this conversion might be lossy (every possible `i64` value cannot fit into `i8`),
/// `truncate` allows value to be truncated to fit into a `i8`, and `or_default` set the
/// value that is assigned when a `i64` is out of range for `i8` and truncation not allowed.
///
/// Truncation happens on the binary level, thus: `10010110` (150 if unsigned, -106 if [signed](https://en.wikipedia.org/wiki/Signed_number_representations)) → `0110` (6).
///
#[mel_function]
pub fn to_i8(value: i64, truncate: bool, or_default: i8) -> i8 {
    if truncate {
        value as i8
    } else {
        use std::convert::TryInto;
        TryInto::<i8>::try_into(value).unwrap_or(or_default)
    }
}

/// Convert stream of `i64` into `i8`.
///
/// As this conversion might be lossy (every possible `i64` value cannot fit into `i8`),
/// `truncate` allows value to be truncated to fit into a `i8`, and `or_default` set the
/// value that is assigned when a `i64` is out of range for `i8` and truncation not allowed.
///
/// Truncation happens on the binary level, thus: `10010110` (150 if unsigned, -106 if [signed](https://en.wikipedia.org/wiki/Signed_number_representations)) → `0110` (6).
///
#[mel_treatment(
    default truncate true
    default or_default 0
    input value Stream<i64>
    output into Stream<i8>
)]
pub async fn to_i8(truncate: bool, or_default: i8) {
    if truncate {
        while let Ok(values) = value
            .recv_many()
            .await
            .map(|values| TryInto::<Vec<i64>>::try_into(values).unwrap())
        {
            check!(
                into.send_many(
                    values
                        .into_iter()
                        .map(|val| val as i8)
                        .collect::<VecDeque<_>>()
                        .into()
                )
                .await
            )
        }
    } else {
        use std::convert::TryInto;
        while let Ok(values) = value
            .recv_many()
            .await
            .map(|values| TryInto::<Vec<i64>>::try_into(values).unwrap())
        {
            check!(
                into.send_many(
                    values
                        .into_iter()
                        .map(|val| TryInto::<i8>::try_into(val).unwrap_or(or_default))
                        .collect::<VecDeque<_>>()
                        .into()
                )
                .await
            )
        }
    }
}

/// Turns `i64` into `i16`.
///
/// As this conversion might be lossy (every possible `i64` value cannot fit into `i16`),
/// `truncate` allows value to be truncated to fit into a `i16`, and `or_default` set the
/// value that is assigned when a `i64` is out of range for `i16` and truncation not allowed.
///
/// Truncation happens on the binary level, thus: `10010110` (150 if unsigned, -106 if [signed](https://en.wikipedia.org/wiki/Signed_number_representations)) → `0110` (6).
///
#[mel_function]
pub fn to_i16(value: i64, truncate: bool, or_default: i16) -> i16 {
    if truncate {
        value as i16
    } else {
        use std::convert::TryInto;
        TryInto::<i16>::try_into(value).unwrap_or(or_default)
    }
}

/// Convert stream of `i64` into `i16`.
///
/// As this conversion might be lossy (every possible `i64` value cannot fit into `i16`),
/// `truncate` allows value to be truncated to fit into a `i16`, and `or_default` set the
/// value that is assigned when a `i64` is out of range for `i16` and truncation not allowed.
///
/// Truncation happens on the binary level, thus: `10010110` (150 if unsigned, -106 if [signed](https://en.wikipedia.org/wiki/Signed_number_representations)) → `0110` (6).
///
#[mel_treatment(
    default truncate true
    default or_default 0
    input value Stream<i64>
    output into Stream<i16>
)]
pub async fn to_i16(truncate: bool, or_default: i16) {
    if truncate {
        while let Ok(values) = value
            .recv_many()
            .await
            .map(|values| TryInto::<Vec<i64>>::try_into(values).unwrap())
        {
            check!(
                into.send_many(
                    values
                        .into_iter()
                        .map(|val| val as i16)
                        .collect::<VecDeque<_>>()
                        .into()
                )
                .await
            )
        }
    } else {
        use std::convert::TryInto;
        while let Ok(values) = value
            .recv_many()
            .await
            .map(|values| TryInto::<Vec<i64>>::try_into(values).unwrap())
        {
            check!(
                into.send_many(
                    values
                        .into_iter()
                        .map(|val| TryInto::<i16>::try_into(val).unwrap_or(or_default))
                        .collect::<VecDeque<_>>()
                        .into()
                )
                .await
            )
        }
    }
}

/// Turns `i64` into `i32`.
///
/// As this conversion might be lossy (every possible `i64` value cannot fit into `i32`),
/// `truncate` allows value to be truncated to fit into a `i32`, and `or_default` set the
/// value that is assigned when a `i64` is out of range for `i32` and truncation not allowed.
///
/// Truncation happens on the binary level, thus: `10010110` (150 if unsigned, -106 if [signed](https://en.wikipedia.org/wiki/Signed_number_representations)) → `0110` (6).
///
#[mel_function]
pub fn to_i32(value: i64, truncate: bool, or_default: i32) -> i32 {
    if truncate {
        value as i32
    } else {
        use std::convert::TryInto;
        TryInto::<i32>::try_into(value).unwrap_or(or_default)
    }
}

/// Convert stream of `i64` into `i32`.
///
/// As this conversion might be lossy (every possible `i64` value cannot fit into `i32`),
/// `truncate` allows value to be truncated to fit into a `i32`, and `or_default` set the
/// value that is assigned when a `i64` is out of range for `i32` and truncation not allowed.
///
/// Truncation happens on the binary level, thus: `10010110` (150 if unsigned, -106 if [signed](https://en.wikipedia.org/wiki/Signed_number_representations)) → `0110` (6).
///
#[mel_treatment(
    default truncate true
    default or_default 0
    input value Stream<i64>
    output into Stream<i32>
)]
pub async fn to_i32(truncate: bool, or_default: i32) {
    if truncate {
        while let Ok(values) = value
            .recv_many()
            .await
            .map(|values| TryInto::<Vec<i64>>::try_into(values).unwrap())
        {
            check!(
                into.send_many(
                    values
                        .into_iter()
                        .map(|val| val as i32)
                        .collect::<VecDeque<_>>()
                        .into()
                )
                .await
            )
        }
    } else {
        use std::convert::TryInto;
        while let Ok(values) = value
            .recv_many()
            .await
            .map(|values| TryInto::<Vec<i64>>::try_into(values).unwrap())
        {
            check!(
                into.send_many(
                    values
                        .into_iter()
                        .map(|val| TryInto::<i32>::try_into(val).unwrap_or(or_default))
                        .collect::<VecDeque<_>>()
                        .into()
                )
                .await
            )
        }
    }
}
