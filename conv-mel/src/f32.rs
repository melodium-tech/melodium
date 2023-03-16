use melodium_core::*;
use melodium_macro::{check, mel_function, mel_treatment};

/// Turns `f32` stream into `void` one.
#[mel_treatment(
    input value Stream<f32>
    output iter Stream<void>
)]
pub async fn to_void() {
    while let Ok(values) = value.recv_f32().await {
        check!(iter.send_void(vec![(); values.len()]).await)
    }
}

/// Convert stream of `f32` into `f64`.
///
/// Every `f32` is fitted into the closest `f64`.
/// Positive and negative infinity are conserved, as well as not-a-number state.
/// If overflowing, infinity of the same sign is used.
#[mel_treatment(
    input value Stream<f32>
    output into Stream<f64>
)]
pub async fn to_f64() {
    while let Ok(values) = value.recv_f32().await {
        check!(
            into.send_f64(values.into_iter().map(|val| val as f64).collect())
                .await
        )
    }
}

/// Convert stream of `f32` into `u8`.
///
/// Every `f32` is truncated to fit into the `u8`, and in case floating-point value does
/// not describe a real number:
/// - `pos_infinity` is used when `f32` is a positive infinity,
/// - `neg_infinity` is used when `f32` is a negative infinity,
/// - `nan` is used when `f32` is not a number.
#[mel_treatment(
    default pos_infinity 255
    default neg_infinity 0
    default nan 0
    input value Stream<f32>
    output into Stream<u8>
)]
pub async fn to_u8(pos_infinity: u8, neg_infinity: u8, nan: u8) {
    while let Ok(numbers) = value.recv_f32().await {
        check!(
            into.send_u8(
                numbers
                    .into_iter()
                    .map(|number| if number.is_finite() {
                        number as u8
                    } else if number.is_nan() {
                        nan
                    } else if number.is_sign_positive() {
                        pos_infinity
                    } else
                    /*if number.is_sign_negative()*/
                    {
                        neg_infinity
                    })
                    .collect()
            )
            .await
        )
    }
}

/// Convert stream of `f32` into `u16`.
///
/// Every `f32` is truncated to fit into the `u16`, and in case floating-point value does
/// not describe a real number:
/// - `pos_infinity` is used when `f32` is a positive infinity,
/// - `neg_infinity` is used when `f32` is a negative infinity,
/// - `nan` is used when `f32` is not a number.
#[mel_treatment(
    default pos_infinity 65535
    default neg_infinity 0
    default nan 0
    input value Stream<f32>
    output into Stream<u16>
)]
pub async fn to_u16(pos_infinity: u16, neg_infinity: u16, nan: u16) {
    while let Ok(numbers) = value.recv_f32().await {
        check!(
            into.send_u16(
                numbers
                    .into_iter()
                    .map(|number| if number.is_finite() {
                        number as u16
                    } else if number.is_nan() {
                        nan
                    } else if number.is_sign_positive() {
                        pos_infinity
                    } else
                    /*if number.is_sign_negative()*/
                    {
                        neg_infinity
                    })
                    .collect()
            )
            .await
        )
    }
}

/// Convert stream of `f32` into `u32`.
///
/// Every `f32` is truncated to fit into the `u32`, and in case floating-point value does
/// not describe a real number:
/// - `pos_infinity` is used when `f32` is a positive infinity,
/// - `neg_infinity` is used when `f32` is a negative infinity,
/// - `nan` is used when `f32` is not a number.
#[mel_treatment(
    default pos_infinity 4294967295
    default neg_infinity 0
    default nan 0
    input value Stream<f32>
    output into Stream<u32>
)]
pub async fn to_u32(pos_infinity: u32, neg_infinity: u32, nan: u32) {
    while let Ok(numbers) = value.recv_f32().await {
        check!(
            into.send_u32(
                numbers
                    .into_iter()
                    .map(|number| if number.is_finite() {
                        number as u32
                    } else if number.is_nan() {
                        nan
                    } else if number.is_sign_positive() {
                        pos_infinity
                    } else
                    /*if number.is_sign_negative()*/
                    {
                        neg_infinity
                    })
                    .collect()
            )
            .await
        )
    }
}

/// Convert stream of `f32` into `u64`.
///
/// Every `f32` is truncated to fit into the `u64`, and in case floating-point value does
/// not describe a real number:
/// - `pos_infinity` is used when `f32` is a positive infinity,
/// - `neg_infinity` is used when `f32` is a negative infinity,
/// - `nan` is used when `f32` is not a number.
#[mel_treatment(
    default pos_infinity 18446744073709551615
    default neg_infinity 0
    default nan 0
    input value Stream<f32>
    output into Stream<u64>
)]
pub async fn to_u64(pos_infinity: u64, neg_infinity: u64, nan: u64) {
    while let Ok(numbers) = value.recv_f32().await {
        check!(
            into.send_u64(
                numbers
                    .into_iter()
                    .map(|number| if number.is_finite() {
                        number as u64
                    } else if number.is_nan() {
                        nan
                    } else if number.is_sign_positive() {
                        pos_infinity
                    } else
                    /*if number.is_sign_negative()*/
                    {
                        neg_infinity
                    })
                    .collect()
            )
            .await
        )
    }
}

/// Convert stream of `f32` into `u128`.
///
/// Every `f32` is truncated to fit into the `u128`, and in case floating-point value does
/// not describe a real number:
/// - `pos_infinity` is used when `f32` is a positive infinity,
/// - `neg_infinity` is used when `f32` is a negative infinity,
/// - `nan` is used when `f32` is not a number.
#[mel_treatment(
    default pos_infinity 340282366920938463463374607431768211455
    default neg_infinity 0
    default nan 0
    input value Stream<f32>
    output into Stream<u128>
)]
pub async fn to_u128(pos_infinity: u128, neg_infinity: u128, nan: u128) {
    while let Ok(numbers) = value.recv_f32().await {
        check!(
            into.send_u128(
                numbers
                    .into_iter()
                    .map(|number| if number.is_finite() {
                        number as u128
                    } else if number.is_nan() {
                        nan
                    } else if number.is_sign_positive() {
                        pos_infinity
                    } else
                    /*if number.is_sign_negative()*/
                    {
                        neg_infinity
                    })
                    .collect()
            )
            .await
        )
    }
}

/// Convert stream of `f32` into `i8`.
///
/// Every `f32` is truncated to fit into the `i8`, and in case floating-point value does
/// not describe a real number:
/// - `pos_infinity` is used when `f32` is a positive infinity,
/// - `neg_infinity` is used when `f32` is a negative infinity,
/// - `nan` is used when `f32` is not a number.
#[mel_treatment(
    default pos_infinity 127
    default neg_infinity -128
    default nan 0
    input value Stream<f32>
    output into Stream<i8>
)]
pub async fn to_i8(pos_infinity: i8, neg_infinity: i8, nan: i8) {
    while let Ok(numbers) = value.recv_f32().await {
        check!(
            into.send_i8(
                numbers
                    .into_iter()
                    .map(|number| if number.is_finite() {
                        number as i8
                    } else if number.is_nan() {
                        nan
                    } else if number.is_sign_positive() {
                        pos_infinity
                    } else
                    /*if number.is_sign_negative()*/
                    {
                        neg_infinity
                    })
                    .collect()
            )
            .await
        )
    }
}

/// Convert stream of `f32` into `i16`.
///
/// Every `f32` is truncated to fit into the `i16`, and in case floating-point value does
/// not describe a real number:
/// - `pos_infinity` is used when `f32` is a positive infinity,
/// - `neg_infinity` is used when `f32` is a negative infinity,
/// - `nan` is used when `f32` is not a number.
#[mel_treatment(
    default pos_infinity 32767
    default neg_infinity -32768
    default nan 0
    input value Stream<f32>
    output into Stream<i16>
)]
pub async fn to_i16(pos_infinity: i16, neg_infinity: i16, nan: i16) {
    while let Ok(numbers) = value.recv_f32().await {
        check!(
            into.send_i16(
                numbers
                    .into_iter()
                    .map(|number| if number.is_finite() {
                        number as i16
                    } else if number.is_nan() {
                        nan
                    } else if number.is_sign_positive() {
                        pos_infinity
                    } else
                    /*if number.is_sign_negative()*/
                    {
                        neg_infinity
                    })
                    .collect()
            )
            .await
        )
    }
}

/// Convert stream of `f32` into `i32`.
///
/// Every `f32` is truncated to fit into the `i32`, and in case floating-point value does
/// not describe a real number:
/// - `pos_infinity` is used when `f32` is a positive infinity,
/// - `neg_infinity` is used when `f32` is a negative infinity,
/// - `nan` is used when `f32` is not a number.
#[mel_treatment(
    default pos_infinity 2147483647
    default neg_infinity -2147483648
    default nan 0
    input value Stream<f32>
    output into Stream<i32>
)]
pub async fn to_i32(pos_infinity: i32, neg_infinity: i32, nan: i32) {
    while let Ok(numbers) = value.recv_f32().await {
        check!(
            into.send_i32(
                numbers
                    .into_iter()
                    .map(|number| if number.is_finite() {
                        number as i32
                    } else if number.is_nan() {
                        nan
                    } else if number.is_sign_positive() {
                        pos_infinity
                    } else
                    /*if number.is_sign_negative()*/
                    {
                        neg_infinity
                    })
                    .collect()
            )
            .await
        )
    }
}

/// Convert stream of `f32` into `i64`.
///
/// Every `f32` is truncated to fit into the `i64`, and in case floating-point value does
/// not describe a real number:
/// - `pos_infinity` is used when `f32` is a positive infinity,
/// - `neg_infinity` is used when `f32` is a negative infinity,
/// - `nan` is used when `f32` is not a number.
#[mel_treatment(
    default pos_infinity 9223372036854775807
    default neg_infinity -9223372036854775808
    default nan 0
    input value Stream<f32>
    output into Stream<i64>
)]
pub async fn to_i64(pos_infinity: i64, neg_infinity: i64, nan: i64) {
    while let Ok(numbers) = value.recv_f32().await {
        check!(
            into.send_i64(
                numbers
                    .into_iter()
                    .map(|number| if number.is_finite() {
                        number as i64
                    } else if number.is_nan() {
                        nan
                    } else if number.is_sign_positive() {
                        pos_infinity
                    } else
                    /*if number.is_sign_negative()*/
                    {
                        neg_infinity
                    })
                    .collect()
            )
            .await
        )
    }
}

/// Convert stream of `f32` into `i128`.
///
/// Every `f32` is truncated to fit into the `i128`, and in case floating-point value does
/// not describe a real number:
/// - `pos_infinity` is used when `f32` is a positive infinity,
/// - `neg_infinity` is used when `f32` is a negative infinity,
/// - `nan` is used when `f32` is not a number.
#[mel_treatment(
    default pos_infinity 170141183460469231731687303715884105727
    default neg_infinity -170141183460469231731687303715884105728
    default nan 0
    input value Stream<f32>
    output into Stream<i128>
)]
pub async fn to_i128(pos_infinity: i128, neg_infinity: i128, nan: i128) {
    while let Ok(numbers) = value.recv_f32().await {
        check!(
            into.send_i128(
                numbers
                    .into_iter()
                    .map(|number| if number.is_finite() {
                        number as i128
                    } else if number.is_nan() {
                        nan
                    } else if number.is_sign_positive() {
                        pos_infinity
                    } else
                    /*if number.is_sign_negative()*/
                    {
                        neg_infinity
                    })
                    .collect()
            )
            .await
        )
    }
}

/// Turns `f32` stream into `byte` one.
///
/// Each `f32` gets converted into `Vec<byte>`, with each vector containing the `bytes`s of the former scalar `f32` it represents.
#[mel_treatment(
    input value Stream<f32>
    output data Stream<Vec<byte>>
)]
pub async fn to_byte() {
    while let Ok(values) = value.recv_f32().await {
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
