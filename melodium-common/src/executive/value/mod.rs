mod data;
mod traits;

use crate::descriptor::DataType;
pub use data::GetData;

#[derive(Clone, PartialEq, Debug)]
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
        }
    }
}
