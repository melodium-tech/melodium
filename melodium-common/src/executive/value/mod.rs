mod data;
mod traits;

use super::Data;
use crate::descriptor::DataType;
pub use data::GetData;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub enum Value {
    Void(()),

    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),

    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),

    F32(f32),
    F64(f64),

    Bool(bool),
    Byte(u8),
    Char(char),
    String(String),

    Vec(Vec<Value>),
    Option(Option<Box<Value>>),

    Data(Arc<dyn Data>),
}

impl Value {
    pub fn datatype(&self) -> DataType {
        match self {
            Value::Void(_) => DataType::Void,

            Value::I8(_) => DataType::I8,
            Value::I16(_) => DataType::I16,
            Value::I32(_) => DataType::I32,
            Value::I64(_) => DataType::I64,
            Value::I128(_) => DataType::I128,

            Value::U8(_) => DataType::U8,
            Value::U16(_) => DataType::U16,
            Value::U32(_) => DataType::U32,
            Value::U64(_) => DataType::U64,
            Value::U128(_) => DataType::U128,

            Value::F32(_) => DataType::F32,
            Value::F64(_) => DataType::F64,

            Value::Bool(_) => DataType::Bool,
            Value::Byte(_) => DataType::Byte,
            Value::Char(_) => DataType::Char,
            Value::String(_) => DataType::String,

            Value::Option(val) => val
                .as_ref()
                .map(|val| DataType::Option(Box::new(val.datatype())))
                .unwrap_or(DataType::Undetermined),
            Value::Vec(val) => val
                .first()
                .map(|val| DataType::Vec(Box::new(val.datatype())))
                .unwrap_or(DataType::Undetermined),

            Value::Data(obj) => DataType::Data(obj.descriptor()),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Void(l0), Self::Void(r0)) => l0 == r0,
            (Self::I8(l0), Self::I8(r0)) => l0 == r0,
            (Self::I16(l0), Self::I16(r0)) => l0 == r0,
            (Self::I32(l0), Self::I32(r0)) => l0 == r0,
            (Self::I64(l0), Self::I64(r0)) => l0 == r0,
            (Self::I128(l0), Self::I128(r0)) => l0 == r0,
            (Self::U8(l0), Self::U8(r0)) => l0 == r0,
            (Self::U16(l0), Self::U16(r0)) => l0 == r0,
            (Self::U32(l0), Self::U32(r0)) => l0 == r0,
            (Self::U64(l0), Self::U64(r0)) => l0 == r0,
            (Self::U128(l0), Self::U128(r0)) => l0 == r0,
            (Self::F32(l0), Self::F32(r0)) => l0 == r0,
            (Self::F64(l0), Self::F64(r0)) => l0 == r0,
            (Self::Bool(l0), Self::Bool(r0)) => l0 == r0,
            (Self::Byte(l0), Self::Byte(r0)) => l0 == r0,
            (Self::Char(l0), Self::Char(r0)) => l0 == r0,
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            (Self::Vec(l0), Self::Vec(r0)) => l0 == r0,
            (Self::Option(l0), Self::Option(r0)) => l0 == r0,
            (Self::Data(_l0), Self::Data(_r0)) => false,
            _ => false,
        }
    }
}
