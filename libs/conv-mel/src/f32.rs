use melodium_core::*;
use melodium_macro::{check, mel_function, mel_treatment};

/// Turns `f32` stream into `void` one.
#[mel_treatment(
    input value Stream<f32>
    output iter Stream<void>
)]
pub async fn to_void() {
    while let Ok(values) = value.recv_many().await {
        check!(iter.send_many(vec![(); values.len()].into()).await)
    }
}

/// Turns `f32` into `f64`.
///
/// Positive and negative infinity are conserved, as well as not-a-number state.
/// If overflowing, infinity of the same sign is used.
#[mel_function]
pub fn to_f64(value: f32) -> f64 {
    value as f64
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
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| TryInto::<Vec<f32>>::try_into(values).unwrap())
    {
        check!(
            into.send_many(TransmissionValue::F64(
                values.into_iter().map(|val| val as f64).collect()
            ))
            .await
        )
    }
}

/// Turns `f32` into `u8`.
///
/// `f32` is truncated to fit into `u8`, and in case floating-point value does
/// not describe a real number:
/// - `pos_infinity` is used when `f32` is a positive infinity,
/// - `neg_infinity` is used when `f32` is a negative infinity,
/// - `nan` is used when `f32` is not a number.
#[mel_function]
pub fn to_u8(value: f32, pos_infinity: u8, neg_infinity: u8, nan: u8) -> u8 {
    if value.is_finite() {
        value as u8
    } else if value.is_nan() {
        nan
    } else if value.is_sign_positive() {
        pos_infinity
    } else
    /*if number.is_sign_negative()*/
    {
        neg_infinity
    }
}

/// Convert stream of `f32` into `u8`.
///
/// Every `f32` is truncated to fit into `u8`, and in case floating-point value does
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
    while let Ok(numbers) = value
        .recv_many()
        .await
        .map(|values| TryInto::<Vec<f32>>::try_into(values).unwrap())
    {
        check!(
            into.send_many(TransmissionValue::U8(
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
            ))
            .await
        )
    }
}

/// Turns `f32` into `u16`.
///
/// `f32` is truncated to fit into `u16`, and in case floating-point value does
/// not describe a real number:
/// - `pos_infinity` is used when `f32` is a positive infinity,
/// - `neg_infinity` is used when `f32` is a negative infinity,
/// - `nan` is used when `f32` is not a number.
#[mel_function]
pub fn to_u16(value: f32, pos_infinity: u16, neg_infinity: u16, nan: u16) -> u16 {
    if value.is_finite() {
        value as u16
    } else if value.is_nan() {
        nan
    } else if value.is_sign_positive() {
        pos_infinity
    } else
    /*if number.is_sign_negative()*/
    {
        neg_infinity
    }
}

/// Convert stream of `f32` into `u16`.
///
/// Every `f32` is truncated to fit into `u16`, and in case floating-point value does
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
    while let Ok(numbers) = value
        .recv_many()
        .await
        .map(|values| TryInto::<Vec<f32>>::try_into(values).unwrap())
    {
        check!(
            into.send_many(TransmissionValue::U16(
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
            ))
            .await
        )
    }
}

/// Turns `f32` into `u32`.
///
/// `f32` is truncated to fit into `u32`, and in case floating-point value does
/// not describe a real number:
/// - `pos_infinity` is used when `f32` is a positive infinity,
/// - `neg_infinity` is used when `f32` is a negative infinity,
/// - `nan` is used when `f32` is not a number.
#[mel_function]
pub fn to_u32(value: f32, pos_infinity: u32, neg_infinity: u32, nan: u32) -> u32 {
    if value.is_finite() {
        value as u32
    } else if value.is_nan() {
        nan
    } else if value.is_sign_positive() {
        pos_infinity
    } else
    /*if number.is_sign_negative()*/
    {
        neg_infinity
    }
}

/// Convert stream of `f32` into `u32`.
///
/// Every `f32` is truncated to fit into `u32`, and in case floating-point value does
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
    while let Ok(numbers) = value
        .recv_many()
        .await
        .map(|values| TryInto::<Vec<f32>>::try_into(values).unwrap())
    {
        check!(
            into.send_many(TransmissionValue::U32(
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
            ))
            .await
        )
    }
}

/// Turns `f32` into `u64`.
///
/// `f32` is truncated to fit into `u64`, and in case floating-point value does
/// not describe a real number:
/// - `pos_infinity` is used when `f32` is a positive infinity,
/// - `neg_infinity` is used when `f32` is a negative infinity,
/// - `nan` is used when `f32` is not a number.
#[mel_function]
pub fn to_u64(value: f32, pos_infinity: u64, neg_infinity: u64, nan: u64) -> u64 {
    if value.is_finite() {
        value as u64
    } else if value.is_nan() {
        nan
    } else if value.is_sign_positive() {
        pos_infinity
    } else
    /*if number.is_sign_negative()*/
    {
        neg_infinity
    }
}

/// Convert stream of `f32` into `u64`.
///
/// Every `f32` is truncated to fit into `u64`, and in case floating-point value does
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
    while let Ok(numbers) = value
        .recv_many()
        .await
        .map(|values| TryInto::<Vec<f32>>::try_into(values).unwrap())
    {
        check!(
            into.send_many(TransmissionValue::U64(
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
            ))
            .await
        )
    }
}

/// Turns `f32` into `u128`.
///
/// `f32` is truncated to fit into `u128`, and in case floating-point value does
/// not describe a real number:
/// - `pos_infinity` is used when `f32` is a positive infinity,
/// - `neg_infinity` is used when `f32` is a negative infinity,
/// - `nan` is used when `f32` is not a number.
#[mel_function]
pub fn to_u128(value: f32, pos_infinity: u128, neg_infinity: u128, nan: u128) -> u128 {
    if value.is_finite() {
        value as u128
    } else if value.is_nan() {
        nan
    } else if value.is_sign_positive() {
        pos_infinity
    } else
    /*if number.is_sign_negative()*/
    {
        neg_infinity
    }
}

/// Convert stream of `f32` into `u128`.
///
/// Every `f32` is truncated to fit into `u128`, and in case floating-point value does
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
    while let Ok(numbers) = value
        .recv_many()
        .await
        .map(|values| TryInto::<Vec<f32>>::try_into(values).unwrap())
    {
        check!(
            into.send_many(TransmissionValue::U128(
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
            ))
            .await
        )
    }
}

/// Turns `f32` into `i8`.
///
/// `f32` is truncated to fit into `i8`, and in case floating-point value does
/// not describe a real number:
/// - `pos_infinity` is used when `f32` is a positive infinity,
/// - `neg_infinity` is used when `f32` is a negative infinity,
/// - `nan` is used when `f32` is not a number.
#[mel_function]
pub fn to_i8(value: f32, pos_infinity: i8, neg_infinity: i8, nan: i8) -> i8 {
    if value.is_finite() {
        value as i8
    } else if value.is_nan() {
        nan
    } else if value.is_sign_positive() {
        pos_infinity
    } else
    /*if number.is_sign_negative()*/
    {
        neg_infinity
    }
}

/// Convert stream of `f32` into `i8`.
///
/// Every `f32` is truncated to fit into `i8`, and in case floating-point value does
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
    while let Ok(numbers) = value
        .recv_many()
        .await
        .map(|values| TryInto::<Vec<f32>>::try_into(values).unwrap())
    {
        check!(
            into.send_many(TransmissionValue::I8(
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
            ))
            .await
        )
    }
}

/// Turns `f32` into `i16`.
///
/// `f32` is truncated to fit into `i16`, and in case floating-point value does
/// not describe a real number:
/// - `pos_infinity` is used when `f32` is a positive infinity,
/// - `neg_infinity` is used when `f32` is a negative infinity,
/// - `nan` is used when `f32` is not a number.
#[mel_function]
pub fn to_i16(value: f32, pos_infinity: i16, neg_infinity: i16, nan: i16) -> i16 {
    if value.is_finite() {
        value as i16
    } else if value.is_nan() {
        nan
    } else if value.is_sign_positive() {
        pos_infinity
    } else
    /*if number.is_sign_negative()*/
    {
        neg_infinity
    }
}

/// Convert stream of `f32` into `i16`.
///
/// Every `f32` is truncated to fit into `i16`, and in case floating-point value does
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
    while let Ok(numbers) = value
        .recv_many()
        .await
        .map(|values| TryInto::<Vec<f32>>::try_into(values).unwrap())
    {
        check!(
            into.send_many(TransmissionValue::I16(
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
            ))
            .await
        )
    }
}

/// Turns `f32` into `i32`.
///
/// `f32` is truncated to fit into `i32`, and in case floating-point value does
/// not describe a real number:
/// - `pos_infinity` is used when `f32` is a positive infinity,
/// - `neg_infinity` is used when `f32` is a negative infinity,
/// - `nan` is used when `f32` is not a number.
#[mel_function]
pub fn to_i32(value: f32, pos_infinity: i32, neg_infinity: i32, nan: i32) -> i32 {
    if value.is_finite() {
        value as i32
    } else if value.is_nan() {
        nan
    } else if value.is_sign_positive() {
        pos_infinity
    } else
    /*if number.is_sign_negative()*/
    {
        neg_infinity
    }
}

/// Convert stream of `f32` into `i32`.
///
/// Every `f32` is truncated to fit into `i32`, and in case floating-point value does
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
    while let Ok(numbers) = value
        .recv_many()
        .await
        .map(|values| TryInto::<Vec<f32>>::try_into(values).unwrap())
    {
        check!(
            into.send_many(TransmissionValue::I32(
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
            ))
            .await
        )
    }
}

/// Turns `f32` into `i64`.
///
/// `f32` is truncated to fit into `i64`, and in case floating-point value does
/// not describe a real number:
/// - `pos_infinity` is used when `f32` is a positive infinity,
/// - `neg_infinity` is used when `f32` is a negative infinity,
/// - `nan` is used when `f32` is not a number.
#[mel_function]
pub fn to_i64(value: f32, pos_infinity: i64, neg_infinity: i64, nan: i64) -> i64 {
    if value.is_finite() {
        value as i64
    } else if value.is_nan() {
        nan
    } else if value.is_sign_positive() {
        pos_infinity
    } else
    /*if number.is_sign_negative()*/
    {
        neg_infinity
    }
}

/// Convert stream of `f32` into `i64`.
///
/// Every `f32` is truncated to fit into `i64`, and in case floating-point value does
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
    while let Ok(numbers) = value
        .recv_many()
        .await
        .map(|values| TryInto::<Vec<f32>>::try_into(values).unwrap())
    {
        check!(
            into.send_many(TransmissionValue::I64(
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
            ))
            .await
        )
    }
}

/// Turns `f32` into `i128`.
///
/// `f32` is truncated to fit into `i128`, and in case floating-point value does
/// not describe a real number:
/// - `pos_infinity` is used when `f32` is a positive infinity,
/// - `neg_infinity` is used when `f32` is a negative infinity,
/// - `nan` is used when `f32` is not a number.
#[mel_function]
pub fn to_i128(value: f32, pos_infinity: i128, neg_infinity: i128, nan: i128) -> i128 {
    if value.is_finite() {
        value as i128
    } else if value.is_nan() {
        nan
    } else if value.is_sign_positive() {
        pos_infinity
    } else
    /*if number.is_sign_negative()*/
    {
        neg_infinity
    }
}

/// Convert stream of `f32` into `i128`.
///
/// Every `f32` is truncated to fit into `i128`, and in case floating-point value does
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
    while let Ok(numbers) = value
        .recv_many()
        .await
        .map(|values| TryInto::<Vec<f32>>::try_into(values).unwrap())
    {
        check!(
            into.send_many(TransmissionValue::I128(
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
            ))
            .await
        )
    }
}

/// Turns `f32` into `Vec<byte>`.
#[mel_function]
pub fn to_byte(value: f32) -> Vec<byte> {
    value.to_be_bytes().to_vec()
}

/// Turns `f32` stream into `byte` one.
///
/// Each `f32` gets converted into `Vec<byte>`, with each vector containing the `bytes`s of the former scalar `f32` it represents.
#[mel_treatment(
    input value Stream<f32>
    output data Stream<Vec<byte>>
)]
pub async fn to_byte() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| TryInto::<Vec<f32>>::try_into(values).unwrap())
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
