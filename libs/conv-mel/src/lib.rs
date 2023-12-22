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
pub mod string;
pub mod u128;
pub mod u16;
pub mod u32;
pub mod u64;
pub mod u8;

/// Turns any stream into `void` one.
#[mel_treatment(
    generic T
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
#[mel_function(
    generic T
)]
pub fn to_byte(value: T) -> Vec<byte> {
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
            true => vec![1],
            false => vec![0],
        },
        Value::Byte(val) => val.to_be_bytes().iter().map(|v| *v).collect(),
        Value::Char(val) => val.to_string().as_bytes().iter().map(|v| *v).collect(),
        Value::String(val) => val.as_bytes().iter().map(|v| *v).collect(),
        Value::Vec(_vals) => Vec::new(),
        Value::Option(val) => match val {
            Some(val) => to_byte(*val),
            None => Vec::new(),
        },
    }
}

/// Turns data stream into `byte` one.
///
/// Each `bool` gets converted into `Vec<byte>`, with each vector containing the `byte` of the former scalar `bool` it represents.
///
/// ℹ️ A `bool` always corresponds to one `byte`, being `0` if `false` and `1` if `true`.
#[mel_treatment(
    generic T
    input value Stream<T>
    output data Stream<Vec<byte>>
)]
pub async fn to_byte() {
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

mel_package!();
