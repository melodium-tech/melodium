#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

use melodium_core::*;
use melodium_macro::{check, mel_function, mel_package, mel_treatment};

pub mod bool;
pub mod char;
pub mod f32;
pub mod f64;
pub mod i128;
pub mod i16;
pub mod i32;
pub mod i64;
pub mod i8;
//pub mod string;
pub mod u128;
pub mod u16;
pub mod u32;
pub mod u64;
pub mod u8;

/// Turns any data into `void`.
#[mel_function(
    generic T ()
)]
pub fn to_void(_value: T) -> void {
    ()
}

/// Turns any stream into `void` one.
#[mel_treatment(
    generic T ()
    input value Stream<T>
    output iter Stream<void>
)]
pub async fn to_void() {
    while let Ok(values) = value.recv_many().await {
        check!(iter.send_many(vec![(); values.len()].into()).await)
    }
}

/// Turns data into `Vec<byte>`.
///
/// Data element gets converted into `Vec<byte>`, with vector containing the binary form of data it represents.
///
/// ℹ️ While this conversion is infaillible, resulting vector may be empty.
/// Content format and length of vector is totally dependent on data type given, and might not be constant (like for `char` or `string` types).

#[mel_function(
    generic T ()
)]
pub fn to_bytes(value: T) -> Vec<byte> {
    match value {
        Value::Void(_) => Vec::new(),
        Value::I8(val) => val.to_be_bytes().iter().map(|v| *v).collect(),
        Value::I16(val) => val.to_be_bytes().iter().map(|v| *v).collect(),
        Value::I32(val) => val.to_be_bytes().iter().map(|v| *v).collect(),
        Value::I64(val) => val.to_be_bytes().iter().map(|v| *v).collect(),
        Value::I128(val) => val.to_be_bytes().iter().map(|v| *v).collect(),
        Value::U8(val) => val.to_be_bytes().iter().map(|v| *v).collect(),
        Value::U16(val) => val.to_be_bytes().iter().map(|v| *v).collect(),
        Value::U32(val) => val.to_be_bytes().iter().map(|v| *v).collect(),
        Value::U64(val) => val.to_be_bytes().iter().map(|v| *v).collect(),
        Value::U128(val) => val.to_be_bytes().iter().map(|v| *v).collect(),
        Value::F32(val) => val.to_be_bytes().iter().map(|v| *v).collect(),
        Value::F64(val) => val.to_be_bytes().iter().map(|v| *v).collect(),
        Value::Bool(val) => match val {
            true => vec![1u8],
            false => vec![0u8],
        },
        Value::Byte(val) => val.to_be_bytes().iter().map(|v| *v).collect(),
        Value::Char(val) => val.to_string().as_bytes().iter().map(|v| *v).collect(),
        Value::String(val) => val.as_bytes().iter().map(|v| *v).collect(),
        Value::Vec(_vals) => Vec::new(),
        Value::Option(val) => match val {
            Some(val) => to_bytes(*val),
            None => Vec::new(),
        },
    }
}

/// Turns data stream into `Vec<byte>` one.
///
/// Each data element gets converted into `Vec<byte>`, with each vector containing the binary form of data it represents.
///
/// ℹ️ While this conversion is infaillible, resulting vector may be empty.
/// Content format and length of each vector is totally dependent on data type given, and might not be constant (like for `char` or `string` types).
#[mel_treatment(
    generic T ()
    input value Stream<T>
    output data Stream<Vec<byte>>
)]
pub async fn to_bytes() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            data.send_many(TransmissionValue::Other(
                values.into_iter().map(|val| value_to_byte(val)).collect()
            ))
            .await
        )
    }
}

/// Converts any value into byte equivalent.
fn value_to_byte(value: Value) -> Value {
    match value {
        Value::Void(_) => Value::Vec(Vec::new()),
        Value::I8(val) => Value::Vec(val.to_be_bytes().iter().map(|v| Value::Byte(*v)).collect()),
        Value::I16(val) => Value::Vec(val.to_be_bytes().iter().map(|v| Value::Byte(*v)).collect()),
        Value::I32(val) => Value::Vec(val.to_be_bytes().iter().map(|v| Value::Byte(*v)).collect()),
        Value::I64(val) => Value::Vec(val.to_be_bytes().iter().map(|v| Value::Byte(*v)).collect()),
        Value::I128(val) => Value::Vec(val.to_be_bytes().iter().map(|v| Value::Byte(*v)).collect()),
        Value::U8(val) => Value::Vec(val.to_be_bytes().iter().map(|v| Value::Byte(*v)).collect()),
        Value::U16(val) => Value::Vec(val.to_be_bytes().iter().map(|v| Value::Byte(*v)).collect()),
        Value::U32(val) => Value::Vec(val.to_be_bytes().iter().map(|v| Value::Byte(*v)).collect()),
        Value::U64(val) => Value::Vec(val.to_be_bytes().iter().map(|v| Value::Byte(*v)).collect()),
        Value::U128(val) => Value::Vec(val.to_be_bytes().iter().map(|v| Value::Byte(*v)).collect()),
        Value::F32(val) => Value::Vec(val.to_be_bytes().iter().map(|v| Value::Byte(*v)).collect()),
        Value::F64(val) => Value::Vec(val.to_be_bytes().iter().map(|v| Value::Byte(*v)).collect()),
        Value::Bool(val) => Value::Vec(match val {
            true => vec![Value::Byte(1)],
            false => vec![Value::Byte(0)],
        }),
        Value::Byte(val) => Value::Vec(val.to_be_bytes().iter().map(|v| Value::Byte(*v)).collect()),
        Value::Char(val) => Value::Vec(
            val.to_string()
                .as_bytes()
                .iter()
                .map(|v| Value::Byte(*v))
                .collect(),
        ),
        Value::String(val) => Value::Vec(val.as_bytes().iter().map(|v| Value::Byte(*v)).collect()),
        Value::Vec(vals) => Value::Vec(vals.into_iter().map(|val| value_to_byte(val)).collect()),
        Value::Option(val) => match val {
            Some(val) => value_to_byte(*val),
            None => Value::Vec(Vec::new()),
        },
    }
}

/// Turns any data into `i8`.
#[mel_function(
    generic T (ToI8)
)]
pub fn to_i8(value: T) -> i8 {
    value.to_i8()
}

/// Turns stream into `i8` one.
///
/// This treatment manages infaillible conversions to `i8` data type.
#[mel_treatment(
    generic T (ToI8)
    input value Stream<T>
    output into Stream<i8>
)]
pub async fn to_i8() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            into.send_many(TransmissionValue::I8(
                values.into_iter().map(|val| val.to_i8()).collect()
            ))
            .await
        )
    }
}

/// Turns any data into `i16`.
#[mel_function(
    generic T (ToI16)
)]
pub fn to_i16(value: T) -> i16 {
    value.to_i16()
}

/// Turns stream into `i16` one.
///
/// This treatment manages infaillible conversions to `i16` data type.
#[mel_treatment(
    generic T (ToI16)
    input value Stream<T>
    output into Stream<i16>
)]
pub async fn to_i16() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            into.send_many(TransmissionValue::I16(
                values.into_iter().map(|val| val.to_i16()).collect()
            ))
            .await
        )
    }
}

/// Turns any data into `i32`.
#[mel_function(
    generic T (ToI32)
)]
pub fn to_i32(value: T) -> i32 {
    value.to_i32()
}

/// Turns stream into `i32` one.
///
/// This treatment manages infaillible conversions to `i32` data type.
#[mel_treatment(
    generic T (ToI32)
    input value Stream<T>
    output into Stream<i32>
)]
pub async fn to_i32() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            into.send_many(TransmissionValue::I32(
                values.into_iter().map(|val| val.to_i32()).collect()
            ))
            .await
        )
    }
}

/// Turns any data into `i64`.
#[mel_function(
    generic T (ToI64)
)]
pub fn to_i64(value: T) -> i64 {
    value.to_i64()
}

/// Turns stream into `i64` one.
///
/// This treatment manages infaillible conversions to `i64` data type.
#[mel_treatment(
    generic T (ToI64)
    input value Stream<T>
    output into Stream<i64>
)]
pub async fn to_i64() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            into.send_many(TransmissionValue::I64(
                values.into_iter().map(|val| val.to_i64()).collect()
            ))
            .await
        )
    }
}

/// Turns any data into `i128`.
#[mel_function(
    generic T (ToI128)
)]
pub fn to_i128(value: T) -> i128 {
    value.to_i128()
}

/// Turns stream into `i128` one.
///
/// This treatment manages infaillible conversions to `i128` data type.
#[mel_treatment(
    generic T (ToI128)
    input value Stream<T>
    output into Stream<i128>
)]
pub async fn to_i128() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            into.send_many(TransmissionValue::I128(
                values.into_iter().map(|val| val.to_i128()).collect()
            ))
            .await
        )
    }
}

/// Turns any data into `u8`.
#[mel_function(
    generic T (ToU8)
)]
pub fn to_u8(value: T) -> u8 {
    value.to_u8()
}

/// Turns stream into `u8` one.
///
/// This treatment manages infaillible conversions to `u8` data type.
#[mel_treatment(
    generic T (ToU8)
    input value Stream<T>
    output into Stream<u8>
)]
pub async fn to_u8() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            into.send_many(TransmissionValue::U8(
                values.into_iter().map(|val| val.to_u8()).collect()
            ))
            .await
        )
    }
}

/// Turns any data into `u16`.
#[mel_function(
    generic T (ToU16)
)]
pub fn to_u16(value: T) -> u16 {
    value.to_u16()
}

/// Turns stream into `u16` one.
///
/// This treatment manages infaillible conversions to `u16` data type.
#[mel_treatment(
    generic T (ToU16)
    input value Stream<T>
    output into Stream<u16>
)]
pub async fn to_u16() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            into.send_many(TransmissionValue::U16(
                values.into_iter().map(|val| val.to_u16()).collect()
            ))
            .await
        )
    }
}

/// Turns any data into `u32`.
#[mel_function(
    generic T (ToU32)
)]
pub fn to_u32(value: T) -> u32 {
    value.to_u32()
}

/// Turns stream into `u32` one.
///
/// This treatment manages infaillible conversions to `u32` data type.
#[mel_treatment(
    generic T (ToU32)
    input value Stream<T>
    output into Stream<u32>
)]
pub async fn to_u32() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            into.send_many(TransmissionValue::U32(
                values.into_iter().map(|val| val.to_u32()).collect()
            ))
            .await
        )
    }
}

/// Turns any data into `u64`.
#[mel_function(
    generic T (ToU64)
)]
pub fn to_u64(value: T) -> u64 {
    value.to_u64()
}

/// Turns stream into `u64` one.
///
/// This treatment manages infaillible conversions to `u64` data type.
#[mel_treatment(
    generic T (ToU64)
    input value Stream<T>
    output into Stream<u64>
)]
pub async fn to_u64() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            into.send_many(TransmissionValue::U64(
                values.into_iter().map(|val| val.to_u64()).collect()
            ))
            .await
        )
    }
}

/// Turns any data into `u128`.
#[mel_function(
    generic T (ToU128)
)]
pub fn to_u128(value: T) -> u128 {
    value.to_u128()
}

/// Turns stream into `u128` one.
///
/// This treatment manages infaillible conversions to `u128` data type.
#[mel_treatment(
    generic T (ToU128)
    input value Stream<T>
    output into Stream<u128>
)]
pub async fn to_u128() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            into.send_many(TransmissionValue::U128(
                values.into_iter().map(|val| val.to_u128()).collect()
            ))
            .await
        )
    }
}

/// Turns any data into `f32`.
#[mel_function(
    generic T (ToF32)
)]
pub fn to_f32(value: T) -> f32 {
    value.to_f32()
}

/// Turns stream into `f32` one.
///
/// This treatment manages infaillible conversions to `f32` data type.
#[mel_treatment(
    generic T (ToF32)
    input value Stream<T>
    output into Stream<f32>
)]
pub async fn to_f32() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            into.send_many(TransmissionValue::F32(
                values.into_iter().map(|val| val.to_f32()).collect()
            ))
            .await
        )
    }
}

/// Turns any data into `f64`.
#[mel_function(
    generic T (ToF64)
)]
pub fn to_f64(value: T) -> f64 {
    value.to_f64()
}

/// Turns stream into `f64` one.
///
/// This treatment manages infaillible conversions to `f64` data type.
#[mel_treatment(
    generic T (ToF64)
    input value Stream<T>
    output into Stream<f64>
)]
pub async fn to_f64() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            into.send_many(TransmissionValue::F64(
                values.into_iter().map(|val| val.to_f64()).collect()
            ))
            .await
        )
    }
}

/// Turns any data into `bool`.
#[mel_function(
    generic T (ToBool)
)]
pub fn to_bool(value: T) -> bool {
    value.to_bool()
}

/// Turns stream into `bool` one.
///
/// This treatment manages infaillible conversions to `bool` data type.
#[mel_treatment(
    generic T (ToBool)
    input value Stream<T>
    output into Stream<bool>
)]
pub async fn to_bool() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            into.send_many(TransmissionValue::Bool(
                values.into_iter().map(|val| val.to_bool()).collect()
            ))
            .await
        )
    }
}

/// Turns any data into `byte`.
/*#[mel_function(
    generic T (ToByte)
)]
pub fn to_byte(value: T) -> byte {
    value.to_byte()
}*/

/// Turns stream into `byte` one.
///
/// This treatment manages infaillible conversions to `byte` data type.
#[mel_treatment(
    generic T (ToByte)
    input value Stream<T>
    output into Stream<byte>
)]
pub async fn to_byte() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            into.send_many(TransmissionValue::Byte(
                values.into_iter().map(|val| val.to_byte()).collect()
            ))
            .await
        )
    }
}

/// Turns any data into `char`.
#[mel_function(
    generic T (ToChar)
)]
pub fn to_char(value: T) -> char {
    value.to_char()
}

/// Turns stream into `char` one.
///
/// This treatment manages infaillible conversions to `char` data type.
#[mel_treatment(
    generic T (ToChar)
    input value Stream<T>
    output into Stream<char>
)]
pub async fn to_char() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            into.send_many(TransmissionValue::Char(
                values.into_iter().map(|val| val.to_char()).collect()
            ))
            .await
        )
    }
}

/// Turns any data into `string`.
#[mel_function(
    generic T (ToString)
)]
pub fn to_string(value: T) -> string {
    DataTrait::to_string(&value)
}

/// Turns stream into `string` one.
///
/// This treatment manages infaillible conversions to `string` data type.
#[mel_treatment(
    generic T (ToString)
    input value Stream<T>
    output into Stream<string>
)]
pub async fn to_string() {
    while let Ok(values) = value
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            into.send_many(TransmissionValue::String(
                values
                    .into_iter()
                    .map(|val| DataTrait::to_string(&val))
                    .collect()
            ))
            .await
        )
    }
}

mel_package!();
