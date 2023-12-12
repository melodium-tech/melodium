use super::Value;
use std::collections::VecDeque;
use std::convert::TryInto;

#[derive(Debug, Clone)]
pub enum TransmissionError {
    NoReceiver,
    EverythingClosed,
    NoData,
}

pub type SendResult = Result<(), TransmissionError>;
pub type RecvResult<T> = Result<T, TransmissionError>;

#[derive(Clone, PartialEq, Debug)]
pub enum TransmissionValue {
    Void(VecDeque<()>),

    I8(VecDeque<i8>),
    I16(VecDeque<i16>),
    I32(VecDeque<i32>),
    I64(VecDeque<i64>),
    I128(VecDeque<i128>),

    U8(VecDeque<u8>),
    U16(VecDeque<u16>),
    U32(VecDeque<u32>),
    U64(VecDeque<u64>),
    U128(VecDeque<u128>),

    F32(VecDeque<f32>),
    F64(VecDeque<f64>),

    Bool(VecDeque<bool>),
    Byte(VecDeque<u8>),
    Char(VecDeque<char>),
    String(VecDeque<String>),

    VecVoid(VecDeque<Vec<()>>),

    VecI8(VecDeque<Vec<i8>>),
    VecI16(VecDeque<Vec<i16>>),
    VecI32(VecDeque<Vec<i32>>),
    VecI64(VecDeque<Vec<i64>>),
    VecI128(VecDeque<Vec<i128>>),

    VecU8(VecDeque<Vec<u8>>),
    VecU16(VecDeque<Vec<u16>>),
    VecU32(VecDeque<Vec<u32>>),
    VecU64(VecDeque<Vec<u64>>),
    VecU128(VecDeque<Vec<u128>>),

    VecF32(VecDeque<Vec<f32>>),
    VecF64(VecDeque<Vec<f64>>),

    VecBool(VecDeque<Vec<bool>>),
    VecByte(VecDeque<Vec<u8>>),
    VecChar(VecDeque<Vec<char>>),
    VecString(VecDeque<Vec<String>>),
}

impl TransmissionValue {
    pub fn new(value: Value) -> Self {
        match value {
            Value::Void(value) => TransmissionValue::Void({
                let mut vec = VecDeque::new();
                vec.push_back(value);
                vec
            }),
            Value::I8(value) => TransmissionValue::I8({
                let mut vec = VecDeque::new();
                vec.push_back(value);
                vec
            }),
            Value::I16(value) => TransmissionValue::I16({
                let mut vec = VecDeque::new();
                vec.push_back(value);
                vec
            }),
            Value::I32(value) => TransmissionValue::I32({
                let mut vec = VecDeque::new();
                vec.push_back(value);
                vec
            }),
            Value::I64(value) => TransmissionValue::I64({
                let mut vec = VecDeque::new();
                vec.push_back(value);
                vec
            }),
            Value::I128(value) => TransmissionValue::I128({
                let mut vec = VecDeque::new();
                vec.push_back(value);
                vec
            }),

            Value::U8(value) => TransmissionValue::U8({
                let mut vec = VecDeque::new();
                vec.push_back(value);
                vec
            }),
            Value::U16(value) => TransmissionValue::U16({
                let mut vec = VecDeque::new();
                vec.push_back(value);
                vec
            }),
            Value::U32(value) => TransmissionValue::U32({
                let mut vec = VecDeque::new();
                vec.push_back(value);
                vec
            }),
            Value::U64(value) => TransmissionValue::U64({
                let mut vec = VecDeque::new();
                vec.push_back(value);
                vec
            }),
            Value::U128(value) => TransmissionValue::U128({
                let mut vec = VecDeque::new();
                vec.push_back(value);
                vec
            }),

            Value::F32(value) => TransmissionValue::F32({
                let mut vec = VecDeque::new();
                vec.push_back(value);
                vec
            }),
            Value::F64(value) => TransmissionValue::F64({
                let mut vec = VecDeque::new();
                vec.push_back(value);
                vec
            }),

            Value::Bool(value) => TransmissionValue::Bool({
                let mut vec = VecDeque::new();
                vec.push_back(value);
                vec
            }),
            Value::Byte(value) => TransmissionValue::Byte({
                let mut vec = VecDeque::new();
                vec.push_back(value);
                vec
            }),
            Value::Char(value) => TransmissionValue::Char({
                let mut vec = VecDeque::new();
                vec.push_back(value);
                vec
            }),
            Value::String(value) => TransmissionValue::String({
                let mut vec = VecDeque::new();
                vec.push_back(value);
                vec
            }),

            Value::VecVoid(value) => TransmissionValue::VecVoid({
                let mut vec = VecDeque::new();
                vec.push_back(value);
                vec
            }),

            Value::VecI8(value) => TransmissionValue::VecI8({
                let mut vec = VecDeque::new();
                vec.push_back(value);
                vec
            }),
            Value::VecI16(value) => TransmissionValue::VecI16({
                let mut vec = VecDeque::new();
                vec.push_back(value);
                vec
            }),
            Value::VecI32(value) => TransmissionValue::VecI32({
                let mut vec = VecDeque::new();
                vec.push_back(value);
                vec
            }),
            Value::VecI64(value) => TransmissionValue::VecI64({
                let mut vec = VecDeque::new();
                vec.push_back(value);
                vec
            }),
            Value::VecI128(value) => TransmissionValue::VecI128({
                let mut vec = VecDeque::new();
                vec.push_back(value);
                vec
            }),

            Value::VecU8(value) => TransmissionValue::VecU8({
                let mut vec = VecDeque::new();
                vec.push_back(value);
                vec
            }),
            Value::VecU16(value) => TransmissionValue::VecU16({
                let mut vec = VecDeque::new();
                vec.push_back(value);
                vec
            }),
            Value::VecU32(value) => TransmissionValue::VecU32({
                let mut vec = VecDeque::new();
                vec.push_back(value);
                vec
            }),
            Value::VecU64(value) => TransmissionValue::VecU64({
                let mut vec = VecDeque::new();
                vec.push_back(value);
                vec
            }),
            Value::VecU128(value) => TransmissionValue::VecU128({
                let mut vec = VecDeque::new();
                vec.push_back(value);
                vec
            }),

            Value::VecF32(value) => TransmissionValue::VecF32({
                let mut vec = VecDeque::new();
                vec.push_back(value);
                vec
            }),
            Value::VecF64(value) => TransmissionValue::VecF64({
                let mut vec = VecDeque::new();
                vec.push_back(value);
                vec
            }),

            Value::VecBool(value) => TransmissionValue::VecBool({
                let mut vec = VecDeque::new();
                vec.push_back(value);
                vec
            }),
            Value::VecByte(value) => TransmissionValue::VecByte({
                let mut vec = VecDeque::new();
                vec.push_back(value);
                vec
            }),
            Value::VecChar(value) => TransmissionValue::VecChar({
                let mut vec = VecDeque::new();
                vec.push_back(value);
                vec
            }),
            Value::VecString(value) => TransmissionValue::VecString({
                let mut vec = VecDeque::new();
                vec.push_back(value);
                vec
            }),
        }
    }

    pub fn append(&mut self, values: TransmissionValue) {
        match (self, values) {
            (TransmissionValue::Void(data), TransmissionValue::Void(mut values)) => {
                data.append(&mut values)
            }
            (TransmissionValue::I8(data), TransmissionValue::I8(mut values)) => {
                data.append(&mut values)
            }
            (TransmissionValue::I16(data), TransmissionValue::I16(mut values)) => {
                data.append(&mut values)
            }
            (TransmissionValue::I32(data), TransmissionValue::I32(mut values)) => {
                data.append(&mut values)
            }
            (TransmissionValue::I64(data), TransmissionValue::I64(mut values)) => {
                data.append(&mut values)
            }
            (TransmissionValue::I128(data), TransmissionValue::I128(mut values)) => {
                data.append(&mut values)
            }

            (TransmissionValue::U8(data), TransmissionValue::U8(mut values)) => {
                data.append(&mut values)
            }
            (TransmissionValue::U16(data), TransmissionValue::U16(mut values)) => {
                data.append(&mut values)
            }
            (TransmissionValue::U32(data), TransmissionValue::U32(mut values)) => {
                data.append(&mut values)
            }
            (TransmissionValue::U64(data), TransmissionValue::U64(mut values)) => {
                data.append(&mut values)
            }
            (TransmissionValue::U128(data), TransmissionValue::U128(mut values)) => {
                data.append(&mut values)
            }

            (TransmissionValue::F32(data), TransmissionValue::F32(mut values)) => {
                data.append(&mut values)
            }
            (TransmissionValue::F64(data), TransmissionValue::F64(mut values)) => {
                data.append(&mut values)
            }

            (TransmissionValue::Bool(data), TransmissionValue::Bool(mut values)) => {
                data.append(&mut values)
            }
            (TransmissionValue::Byte(data), TransmissionValue::Byte(mut values)) => {
                data.append(&mut values)
            }
            (TransmissionValue::Char(data), TransmissionValue::Char(mut values)) => {
                data.append(&mut values)
            }
            (TransmissionValue::String(data), TransmissionValue::String(mut values)) => {
                data.append(&mut values)
            }

            (TransmissionValue::VecVoid(data), TransmissionValue::VecVoid(mut values)) => {
                data.append(&mut values)
            }

            (TransmissionValue::VecI8(data), TransmissionValue::VecI8(mut values)) => {
                data.append(&mut values)
            }
            (TransmissionValue::VecI16(data), TransmissionValue::VecI16(mut values)) => {
                data.append(&mut values)
            }
            (TransmissionValue::VecI32(data), TransmissionValue::VecI32(mut values)) => {
                data.append(&mut values)
            }
            (TransmissionValue::VecI64(data), TransmissionValue::VecI64(mut values)) => {
                data.append(&mut values)
            }
            (TransmissionValue::VecI128(data), TransmissionValue::VecI128(mut values)) => {
                data.append(&mut values)
            }

            (TransmissionValue::VecU8(data), TransmissionValue::VecU8(mut values)) => {
                data.append(&mut values)
            }
            (TransmissionValue::VecU16(data), TransmissionValue::VecU16(mut values)) => {
                data.append(&mut values)
            }
            (TransmissionValue::VecU32(data), TransmissionValue::VecU32(mut values)) => {
                data.append(&mut values)
            }
            (TransmissionValue::VecU64(data), TransmissionValue::VecU64(mut values)) => {
                data.append(&mut values)
            }
            (TransmissionValue::VecU128(data), TransmissionValue::VecU128(mut values)) => {
                data.append(&mut values)
            }

            (TransmissionValue::VecF32(data), TransmissionValue::VecF32(mut values)) => {
                data.append(&mut values)
            }
            (TransmissionValue::VecF64(data), TransmissionValue::VecF64(mut values)) => {
                data.append(&mut values)
            }

            (TransmissionValue::VecBool(data), TransmissionValue::VecBool(mut values)) => {
                data.append(&mut values)
            }
            (TransmissionValue::VecByte(data), TransmissionValue::VecByte(mut values)) => {
                data.append(&mut values)
            }
            (TransmissionValue::VecChar(data), TransmissionValue::VecChar(mut values)) => {
                data.append(&mut values)
            }
            (TransmissionValue::VecString(data), TransmissionValue::VecString(mut values)) => {
                data.append(&mut values)
            }
            _ => panic!("Adding nonmatching values type in transmitter, aborting."),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            TransmissionValue::Void(data) => data.len(),
            TransmissionValue::I8(data) => data.len(),
            TransmissionValue::I16(data) => data.len(),
            TransmissionValue::I32(data) => data.len(),
            TransmissionValue::I64(data) => data.len(),
            TransmissionValue::I128(data) => data.len(),
            TransmissionValue::U8(data) => data.len(),
            TransmissionValue::U16(data) => data.len(),
            TransmissionValue::U32(data) => data.len(),
            TransmissionValue::U64(data) => data.len(),
            TransmissionValue::U128(data) => data.len(),
            TransmissionValue::F32(data) => data.len(),
            TransmissionValue::F64(data) => data.len(),
            TransmissionValue::Bool(data) => data.len(),
            TransmissionValue::Byte(data) => data.len(),
            TransmissionValue::Char(data) => data.len(),
            TransmissionValue::String(data) => data.len(),
            TransmissionValue::VecVoid(data) => data.len(),
            TransmissionValue::VecI8(data) => data.len(),
            TransmissionValue::VecI16(data) => data.len(),
            TransmissionValue::VecI32(data) => data.len(),
            TransmissionValue::VecI64(data) => data.len(),
            TransmissionValue::VecI128(data) => data.len(),
            TransmissionValue::VecU8(data) => data.len(),
            TransmissionValue::VecU16(data) => data.len(),
            TransmissionValue::VecU32(data) => data.len(),
            TransmissionValue::VecU64(data) => data.len(),
            TransmissionValue::VecU128(data) => data.len(),
            TransmissionValue::VecF32(data) => data.len(),
            TransmissionValue::VecF64(data) => data.len(),
            TransmissionValue::VecBool(data) => data.len(),
            TransmissionValue::VecByte(data) => data.len(),
            TransmissionValue::VecChar(data) => data.len(),
            TransmissionValue::VecString(data) => data.len(),
        }
    }

    pub fn pop_front(&mut self) -> Option<Value> {
        match self {
            TransmissionValue::Void(data) => data.pop_front().map(|data| data.into()),
            TransmissionValue::I8(data) => data.pop_front().map(|data| data.into()),
            TransmissionValue::I16(data) => data.pop_front().map(|data| data.into()),
            TransmissionValue::I32(data) => data.pop_front().map(|data| data.into()),
            TransmissionValue::I64(data) => data.pop_front().map(|data| data.into()),
            TransmissionValue::I128(data) => data.pop_front().map(|data| data.into()),
            TransmissionValue::U8(data) => data.pop_front().map(|data| data.into()),
            TransmissionValue::U16(data) => data.pop_front().map(|data| data.into()),
            TransmissionValue::U32(data) => data.pop_front().map(|data| data.into()),
            TransmissionValue::U64(data) => data.pop_front().map(|data| data.into()),
            TransmissionValue::U128(data) => data.pop_front().map(|data| data.into()),
            TransmissionValue::F32(data) => data.pop_front().map(|data| data.into()),
            TransmissionValue::F64(data) => data.pop_front().map(|data| data.into()),
            TransmissionValue::Bool(data) => data.pop_front().map(|data| data.into()),
            TransmissionValue::Byte(data) => data.pop_front().map(|data| Value::Byte(data)),
            TransmissionValue::Char(data) => data.pop_front().map(|data| data.into()),
            TransmissionValue::String(data) => data.pop_front().map(|data| data.into()),
            TransmissionValue::VecVoid(data) => data.pop_front().map(|data| data.into()),
            TransmissionValue::VecI8(data) => data.pop_front().map(|data| data.into()),
            TransmissionValue::VecI16(data) => data.pop_front().map(|data| data.into()),
            TransmissionValue::VecI32(data) => data.pop_front().map(|data| data.into()),
            TransmissionValue::VecI64(data) => data.pop_front().map(|data| data.into()),
            TransmissionValue::VecI128(data) => data.pop_front().map(|data| data.into()),
            TransmissionValue::VecU8(data) => data.pop_front().map(|data| data.into()),
            TransmissionValue::VecU16(data) => data.pop_front().map(|data| data.into()),
            TransmissionValue::VecU32(data) => data.pop_front().map(|data| data.into()),
            TransmissionValue::VecU64(data) => data.pop_front().map(|data| data.into()),
            TransmissionValue::VecU128(data) => data.pop_front().map(|data| data.into()),
            TransmissionValue::VecF32(data) => data.pop_front().map(|data| data.into()),
            TransmissionValue::VecF64(data) => data.pop_front().map(|data| data.into()),
            TransmissionValue::VecBool(data) => data.pop_front().map(|data| data.into()),
            TransmissionValue::VecByte(data) => data.pop_front().map(|data| Value::VecByte(data)),
            TransmissionValue::VecChar(data) => data.pop_front().map(|data| data.into()),
            TransmissionValue::VecString(data) => data.pop_front().map(|data| data.into()),
        }
    }

    pub fn push(&mut self, value: Value) {
        match (self, value) {
            (TransmissionValue::Void(data), Value::Void(value)) => data.push_back(value),
            (TransmissionValue::I8(data), Value::I8(value)) => data.push_back(value),
            (TransmissionValue::I16(data), Value::I16(value)) => data.push_back(value),
            (TransmissionValue::I32(data), Value::I32(value)) => data.push_back(value),
            (TransmissionValue::I64(data), Value::I64(value)) => data.push_back(value),
            (TransmissionValue::I128(data), Value::I128(value)) => data.push_back(value),

            (TransmissionValue::U8(data), Value::U8(value)) => data.push_back(value),
            (TransmissionValue::U16(data), Value::U16(value)) => data.push_back(value),
            (TransmissionValue::U32(data), Value::U32(value)) => data.push_back(value),
            (TransmissionValue::U64(data), Value::U64(value)) => data.push_back(value),
            (TransmissionValue::U128(data), Value::U128(value)) => data.push_back(value),

            (TransmissionValue::F32(data), Value::F32(value)) => data.push_back(value),
            (TransmissionValue::F64(data), Value::F64(value)) => data.push_back(value),

            (TransmissionValue::Bool(data), Value::Bool(value)) => data.push_back(value),
            (TransmissionValue::Byte(data), Value::Byte(value)) => data.push_back(value),
            (TransmissionValue::Char(data), Value::Char(value)) => data.push_back(value),
            (TransmissionValue::String(data), Value::String(value)) => data.push_back(value),

            (TransmissionValue::VecVoid(data), Value::VecVoid(value)) => data.push_back(value),

            (TransmissionValue::VecI8(data), Value::VecI8(value)) => data.push_back(value),
            (TransmissionValue::VecI16(data), Value::VecI16(value)) => data.push_back(value),
            (TransmissionValue::VecI32(data), Value::VecI32(value)) => data.push_back(value),
            (TransmissionValue::VecI64(data), Value::VecI64(value)) => data.push_back(value),
            (TransmissionValue::VecI128(data), Value::VecI128(value)) => data.push_back(value),

            (TransmissionValue::VecU8(data), Value::VecU8(value)) => data.push_back(value),
            (TransmissionValue::VecU16(data), Value::VecU16(value)) => data.push_back(value),
            (TransmissionValue::VecU32(data), Value::VecU32(value)) => data.push_back(value),
            (TransmissionValue::VecU64(data), Value::VecU64(value)) => data.push_back(value),
            (TransmissionValue::VecU128(data), Value::VecU128(value)) => data.push_back(value),

            (TransmissionValue::VecF32(data), Value::VecF32(value)) => data.push_back(value),
            (TransmissionValue::VecF64(data), Value::VecF64(value)) => data.push_back(value),

            (TransmissionValue::VecBool(data), Value::VecBool(value)) => data.push_back(value),
            (TransmissionValue::VecByte(data), Value::VecByte(value)) => data.push_back(value),
            (TransmissionValue::VecChar(data), Value::VecChar(value)) => data.push_back(value),
            (TransmissionValue::VecString(data), Value::VecString(value)) => data.push_back(value),
            _ => panic!("Adding nonmatching value type in transmitter, aborting."),
        }
    }
}

impl Into<VecDeque<Value>> for TransmissionValue {
    fn into(self) -> VecDeque<Value> {
        match self {
            TransmissionValue::Void(data) => data.into_iter().map(|data| data.into()).collect(),
            TransmissionValue::I8(data) => data.into_iter().map(|data| data.into()).collect(),
            TransmissionValue::I16(data) => data.into_iter().map(|data| data.into()).collect(),
            TransmissionValue::I32(data) => data.into_iter().map(|data| data.into()).collect(),
            TransmissionValue::I64(data) => data.into_iter().map(|data| data.into()).collect(),
            TransmissionValue::I128(data) => data.into_iter().map(|data| data.into()).collect(),
            TransmissionValue::U8(data) => data.into_iter().map(|data| data.into()).collect(),
            TransmissionValue::U16(data) => data.into_iter().map(|data| data.into()).collect(),
            TransmissionValue::U32(data) => data.into_iter().map(|data| data.into()).collect(),
            TransmissionValue::U64(data) => data.into_iter().map(|data| data.into()).collect(),
            TransmissionValue::U128(data) => data.into_iter().map(|data| data.into()).collect(),
            TransmissionValue::F32(data) => data.into_iter().map(|data| data.into()).collect(),
            TransmissionValue::F64(data) => data.into_iter().map(|data| data.into()).collect(),
            TransmissionValue::Bool(data) => data.into_iter().map(|data| data.into()).collect(),
            TransmissionValue::Byte(data) => {
                data.into_iter().map(|data| Value::Byte(data)).collect()
            }
            TransmissionValue::Char(data) => data.into_iter().map(|data| data.into()).collect(),
            TransmissionValue::String(data) => data.into_iter().map(|data| data.into()).collect(),
            TransmissionValue::VecVoid(data) => data.into_iter().map(|data| data.into()).collect(),
            TransmissionValue::VecI8(data) => data.into_iter().map(|data| data.into()).collect(),
            TransmissionValue::VecI16(data) => data.into_iter().map(|data| data.into()).collect(),
            TransmissionValue::VecI32(data) => data.into_iter().map(|data| data.into()).collect(),
            TransmissionValue::VecI64(data) => data.into_iter().map(|data| data.into()).collect(),
            TransmissionValue::VecI128(data) => data.into_iter().map(|data| data.into()).collect(),
            TransmissionValue::VecU8(data) => data.into_iter().map(|data| data.into()).collect(),
            TransmissionValue::VecU16(data) => data.into_iter().map(|data| data.into()).collect(),
            TransmissionValue::VecU32(data) => data.into_iter().map(|data| data.into()).collect(),
            TransmissionValue::VecU64(data) => data.into_iter().map(|data| data.into()).collect(),
            TransmissionValue::VecU128(data) => data.into_iter().map(|data| data.into()).collect(),
            TransmissionValue::VecF32(data) => data.into_iter().map(|data| data.into()).collect(),
            TransmissionValue::VecF64(data) => data.into_iter().map(|data| data.into()).collect(),
            TransmissionValue::VecBool(data) => data.into_iter().map(|data| data.into()).collect(),
            TransmissionValue::VecByte(data) => {
                data.into_iter().map(|data| Value::VecByte(data)).collect()
            }
            TransmissionValue::VecChar(data) => data.into_iter().map(|data| data.into()).collect(),
            TransmissionValue::VecString(data) => {
                data.into_iter().map(|data| data.into()).collect()
            }
        }
    }
}
impl Into<Vec<Value>> for TransmissionValue {
    fn into(self) -> Vec<Value> {
        match self {
            TransmissionValue::Void(data) => data.into_iter().map(|data| data.into()).collect(),
            TransmissionValue::I8(data) => data.into_iter().map(|data| data.into()).collect(),
            TransmissionValue::I16(data) => data.into_iter().map(|data| data.into()).collect(),
            TransmissionValue::I32(data) => data.into_iter().map(|data| data.into()).collect(),
            TransmissionValue::I64(data) => data.into_iter().map(|data| data.into()).collect(),
            TransmissionValue::I128(data) => data.into_iter().map(|data| data.into()).collect(),
            TransmissionValue::U8(data) => data.into_iter().map(|data| data.into()).collect(),
            TransmissionValue::U16(data) => data.into_iter().map(|data| data.into()).collect(),
            TransmissionValue::U32(data) => data.into_iter().map(|data| data.into()).collect(),
            TransmissionValue::U64(data) => data.into_iter().map(|data| data.into()).collect(),
            TransmissionValue::U128(data) => data.into_iter().map(|data| data.into()).collect(),
            TransmissionValue::F32(data) => data.into_iter().map(|data| data.into()).collect(),
            TransmissionValue::F64(data) => data.into_iter().map(|data| data.into()).collect(),
            TransmissionValue::Bool(data) => data.into_iter().map(|data| data.into()).collect(),
            TransmissionValue::Byte(data) => {
                data.into_iter().map(|data| Value::Byte(data)).collect()
            }
            TransmissionValue::Char(data) => data.into_iter().map(|data| data.into()).collect(),
            TransmissionValue::String(data) => data.into_iter().map(|data| data.into()).collect(),
            TransmissionValue::VecVoid(data) => data.into_iter().map(|data| data.into()).collect(),
            TransmissionValue::VecI8(data) => data.into_iter().map(|data| data.into()).collect(),
            TransmissionValue::VecI16(data) => data.into_iter().map(|data| data.into()).collect(),
            TransmissionValue::VecI32(data) => data.into_iter().map(|data| data.into()).collect(),
            TransmissionValue::VecI64(data) => data.into_iter().map(|data| data.into()).collect(),
            TransmissionValue::VecI128(data) => data.into_iter().map(|data| data.into()).collect(),
            TransmissionValue::VecU8(data) => data.into_iter().map(|data| data.into()).collect(),
            TransmissionValue::VecU16(data) => data.into_iter().map(|data| data.into()).collect(),
            TransmissionValue::VecU32(data) => data.into_iter().map(|data| data.into()).collect(),
            TransmissionValue::VecU64(data) => data.into_iter().map(|data| data.into()).collect(),
            TransmissionValue::VecU128(data) => data.into_iter().map(|data| data.into()).collect(),
            TransmissionValue::VecF32(data) => data.into_iter().map(|data| data.into()).collect(),
            TransmissionValue::VecF64(data) => data.into_iter().map(|data| data.into()).collect(),
            TransmissionValue::VecBool(data) => data.into_iter().map(|data| data.into()).collect(),
            TransmissionValue::VecByte(data) => {
                data.into_iter().map(|data| Value::VecByte(data)).collect()
            }
            TransmissionValue::VecChar(data) => data.into_iter().map(|data| data.into()).collect(),
            TransmissionValue::VecString(data) => {
                data.into_iter().map(|data| data.into()).collect()
            }
        }
    }
}

impl From<VecDeque<()>> for TransmissionValue {
    fn from(value: VecDeque<()>) -> Self {
        TransmissionValue::Void(value)
    }
}

impl From<Vec<()>> for TransmissionValue {
    fn from(value: Vec<()>) -> Self {
        TransmissionValue::Void(value.into())
    }
}

impl TryInto<VecDeque<()>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<VecDeque<()>, Self::Error> {
        match self {
            TransmissionValue::Void(data) => Ok(data),
            _ => Err(self),
        }
    }
}

impl TryInto<Vec<()>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<Vec<()>, Self::Error> {
        match self {
            TransmissionValue::Void(data) => Ok(data.into()),
            _ => Err(self),
        }
    }
}

impl From<VecDeque<i8>> for TransmissionValue {
    fn from(value: VecDeque<i8>) -> Self {
        TransmissionValue::I8(value)
    }
}

impl From<Vec<i8>> for TransmissionValue {
    fn from(value: Vec<i8>) -> Self {
        TransmissionValue::I8(value.into())
    }
}

impl TryInto<VecDeque<i8>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<VecDeque<i8>, Self::Error> {
        match self {
            TransmissionValue::I8(data) => Ok(data),
            _ => Err(self),
        }
    }
}

impl TryInto<Vec<i8>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<Vec<i8>, Self::Error> {
        match self {
            TransmissionValue::I8(data) => Ok(data.into()),
            _ => Err(self),
        }
    }
}

impl From<VecDeque<i16>> for TransmissionValue {
    fn from(value: VecDeque<i16>) -> Self {
        TransmissionValue::I16(value)
    }
}

impl From<Vec<i16>> for TransmissionValue {
    fn from(value: Vec<i16>) -> Self {
        TransmissionValue::I16(value.into())
    }
}

impl TryInto<VecDeque<i16>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<VecDeque<i16>, Self::Error> {
        match self {
            TransmissionValue::I16(data) => Ok(data),
            _ => Err(self),
        }
    }
}

impl TryInto<Vec<i16>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<Vec<i16>, Self::Error> {
        match self {
            TransmissionValue::I16(data) => Ok(data.into()),
            _ => Err(self),
        }
    }
}

impl From<VecDeque<i32>> for TransmissionValue {
    fn from(value: VecDeque<i32>) -> Self {
        TransmissionValue::I32(value)
    }
}

impl From<Vec<i32>> for TransmissionValue {
    fn from(value: Vec<i32>) -> Self {
        TransmissionValue::I32(value.into())
    }
}

impl TryInto<VecDeque<i32>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<VecDeque<i32>, Self::Error> {
        match self {
            TransmissionValue::I32(data) => Ok(data),
            _ => Err(self),
        }
    }
}

impl TryInto<Vec<i32>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<Vec<i32>, Self::Error> {
        match self {
            TransmissionValue::I32(data) => Ok(data.into()),
            _ => Err(self),
        }
    }
}

impl From<VecDeque<i64>> for TransmissionValue {
    fn from(value: VecDeque<i64>) -> Self {
        TransmissionValue::I64(value)
    }
}

impl From<Vec<i64>> for TransmissionValue {
    fn from(value: Vec<i64>) -> Self {
        TransmissionValue::I64(value.into())
    }
}

impl TryInto<VecDeque<i64>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<VecDeque<i64>, Self::Error> {
        match self {
            TransmissionValue::I64(data) => Ok(data),
            _ => Err(self),
        }
    }
}

impl TryInto<Vec<i64>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<Vec<i64>, Self::Error> {
        match self {
            TransmissionValue::I64(data) => Ok(data.into()),
            _ => Err(self),
        }
    }
}

impl From<VecDeque<i128>> for TransmissionValue {
    fn from(value: VecDeque<i128>) -> Self {
        TransmissionValue::I128(value)
    }
}

impl From<Vec<i128>> for TransmissionValue {
    fn from(value: Vec<i128>) -> Self {
        TransmissionValue::I128(value.into())
    }
}

impl TryInto<VecDeque<i128>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<VecDeque<i128>, Self::Error> {
        match self {
            TransmissionValue::I128(data) => Ok(data),
            _ => Err(self),
        }
    }
}

impl TryInto<Vec<i128>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<Vec<i128>, Self::Error> {
        match self {
            TransmissionValue::I128(data) => Ok(data.into()),
            _ => Err(self),
        }
    }
}

impl From<VecDeque<u8>> for TransmissionValue {
    fn from(value: VecDeque<u8>) -> Self {
        TransmissionValue::U8(value)
    }
}

impl From<Vec<u8>> for TransmissionValue {
    fn from(value: Vec<u8>) -> Self {
        TransmissionValue::U8(value.into())
    }
}

impl TryInto<VecDeque<u8>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<VecDeque<u8>, Self::Error> {
        match self {
            TransmissionValue::U8(data) => Ok(data),
            TransmissionValue::Byte(data) => Ok(data),
            _ => Err(self),
        }
    }
}

impl TryInto<Vec<u8>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<Vec<u8>, Self::Error> {
        match self {
            TransmissionValue::U8(data) => Ok(data.into()),
            TransmissionValue::Byte(data) => Ok(data.into()),
            _ => Err(self),
        }
    }
}

impl From<VecDeque<u16>> for TransmissionValue {
    fn from(value: VecDeque<u16>) -> Self {
        TransmissionValue::U16(value)
    }
}

impl From<Vec<u16>> for TransmissionValue {
    fn from(value: Vec<u16>) -> Self {
        TransmissionValue::U16(value.into())
    }
}

impl TryInto<VecDeque<u16>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<VecDeque<u16>, Self::Error> {
        match self {
            TransmissionValue::U16(data) => Ok(data),
            _ => Err(self),
        }
    }
}

impl TryInto<Vec<u16>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<Vec<u16>, Self::Error> {
        match self {
            TransmissionValue::U16(data) => Ok(data.into()),
            _ => Err(self),
        }
    }
}

impl From<VecDeque<u32>> for TransmissionValue {
    fn from(value: VecDeque<u32>) -> Self {
        TransmissionValue::U32(value)
    }
}

impl From<Vec<u32>> for TransmissionValue {
    fn from(value: Vec<u32>) -> Self {
        TransmissionValue::U32(value.into())
    }
}

impl TryInto<VecDeque<u32>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<VecDeque<u32>, Self::Error> {
        match self {
            TransmissionValue::U32(data) => Ok(data),
            _ => Err(self),
        }
    }
}

impl TryInto<Vec<u32>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<Vec<u32>, Self::Error> {
        match self {
            TransmissionValue::U32(data) => Ok(data.into()),
            _ => Err(self),
        }
    }
}

impl From<VecDeque<u64>> for TransmissionValue {
    fn from(value: VecDeque<u64>) -> Self {
        TransmissionValue::U64(value)
    }
}

impl From<Vec<u64>> for TransmissionValue {
    fn from(value: Vec<u64>) -> Self {
        TransmissionValue::U64(value.into())
    }
}

impl TryInto<VecDeque<u64>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<VecDeque<u64>, Self::Error> {
        match self {
            TransmissionValue::U64(data) => Ok(data),
            _ => Err(self),
        }
    }
}

impl TryInto<Vec<u64>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<Vec<u64>, Self::Error> {
        match self {
            TransmissionValue::U64(data) => Ok(data.into()),
            _ => Err(self),
        }
    }
}

impl From<VecDeque<u128>> for TransmissionValue {
    fn from(value: VecDeque<u128>) -> Self {
        TransmissionValue::U128(value)
    }
}

impl From<Vec<u128>> for TransmissionValue {
    fn from(value: Vec<u128>) -> Self {
        TransmissionValue::U128(value.into())
    }
}

impl TryInto<VecDeque<u128>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<VecDeque<u128>, Self::Error> {
        match self {
            TransmissionValue::U128(data) => Ok(data),
            _ => Err(self),
        }
    }
}

impl TryInto<Vec<u128>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<Vec<u128>, Self::Error> {
        match self {
            TransmissionValue::U128(data) => Ok(data.into()),
            _ => Err(self),
        }
    }
}

impl From<VecDeque<f32>> for TransmissionValue {
    fn from(value: VecDeque<f32>) -> Self {
        TransmissionValue::F32(value)
    }
}

impl From<Vec<f32>> for TransmissionValue {
    fn from(value: Vec<f32>) -> Self {
        TransmissionValue::F32(value.into())
    }
}

impl TryInto<VecDeque<f32>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<VecDeque<f32>, Self::Error> {
        match self {
            TransmissionValue::F32(data) => Ok(data),
            _ => Err(self),
        }
    }
}

impl TryInto<Vec<f32>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<Vec<f32>, Self::Error> {
        match self {
            TransmissionValue::F32(data) => Ok(data.into()),
            _ => Err(self),
        }
    }
}

impl From<VecDeque<f64>> for TransmissionValue {
    fn from(value: VecDeque<f64>) -> Self {
        TransmissionValue::F64(value)
    }
}

impl From<Vec<f64>> for TransmissionValue {
    fn from(value: Vec<f64>) -> Self {
        TransmissionValue::F64(value.into())
    }
}

impl TryInto<VecDeque<f64>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<VecDeque<f64>, Self::Error> {
        match self {
            TransmissionValue::F64(data) => Ok(data),
            _ => Err(self),
        }
    }
}

impl TryInto<Vec<f64>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<Vec<f64>, Self::Error> {
        match self {
            TransmissionValue::F64(data) => Ok(data.into()),
            _ => Err(self),
        }
    }
}

impl From<VecDeque<bool>> for TransmissionValue {
    fn from(value: VecDeque<bool>) -> Self {
        TransmissionValue::Bool(value)
    }
}

impl From<Vec<bool>> for TransmissionValue {
    fn from(value: Vec<bool>) -> Self {
        TransmissionValue::Bool(value.into())
    }
}

impl TryInto<VecDeque<bool>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<VecDeque<bool>, Self::Error> {
        match self {
            TransmissionValue::Bool(data) => Ok(data),
            _ => Err(self),
        }
    }
}

impl TryInto<Vec<bool>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<Vec<bool>, Self::Error> {
        match self {
            TransmissionValue::Bool(data) => Ok(data.into()),
            _ => Err(self),
        }
    }
}

impl From<VecDeque<char>> for TransmissionValue {
    fn from(value: VecDeque<char>) -> Self {
        TransmissionValue::Char(value)
    }
}

impl From<Vec<char>> for TransmissionValue {
    fn from(value: Vec<char>) -> Self {
        TransmissionValue::Char(value.into())
    }
}

impl TryInto<VecDeque<char>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<VecDeque<char>, Self::Error> {
        match self {
            TransmissionValue::Char(data) => Ok(data),
            _ => Err(self),
        }
    }
}

impl TryInto<Vec<char>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<Vec<char>, Self::Error> {
        match self {
            TransmissionValue::Char(data) => Ok(data.into()),
            _ => Err(self),
        }
    }
}

impl From<VecDeque<String>> for TransmissionValue {
    fn from(value: VecDeque<String>) -> Self {
        TransmissionValue::String(value)
    }
}

impl From<Vec<String>> for TransmissionValue {
    fn from(value: Vec<String>) -> Self {
        TransmissionValue::String(value.into())
    }
}

impl TryInto<VecDeque<String>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<VecDeque<String>, Self::Error> {
        match self {
            TransmissionValue::String(data) => Ok(data),
            _ => Err(self),
        }
    }
}

impl TryInto<Vec<String>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<Vec<String>, Self::Error> {
        match self {
            TransmissionValue::String(data) => Ok(data.into()),
            _ => Err(self),
        }
    }
}

impl From<VecDeque<Vec<()>>> for TransmissionValue {
    fn from(value: VecDeque<Vec<()>>) -> Self {
        TransmissionValue::VecVoid(value)
    }
}

impl From<Vec<Vec<()>>> for TransmissionValue {
    fn from(value: Vec<Vec<()>>) -> Self {
        TransmissionValue::VecVoid(value.into())
    }
}

impl TryInto<VecDeque<Vec<()>>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<VecDeque<Vec<()>>, Self::Error> {
        match self {
            TransmissionValue::VecVoid(data) => Ok(data),
            _ => Err(self),
        }
    }
}

impl TryInto<Vec<Vec<()>>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<Vec<Vec<()>>, Self::Error> {
        match self {
            TransmissionValue::VecVoid(data) => Ok(data.into()),
            _ => Err(self),
        }
    }
}

impl From<VecDeque<Vec<i8>>> for TransmissionValue {
    fn from(value: VecDeque<Vec<i8>>) -> Self {
        TransmissionValue::VecI8(value)
    }
}

impl From<Vec<Vec<i8>>> for TransmissionValue {
    fn from(value: Vec<Vec<i8>>) -> Self {
        TransmissionValue::VecI8(value.into())
    }
}

impl TryInto<VecDeque<Vec<i8>>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<VecDeque<Vec<i8>>, Self::Error> {
        match self {
            TransmissionValue::VecI8(data) => Ok(data),
            _ => Err(self),
        }
    }
}

impl TryInto<Vec<Vec<i8>>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<Vec<Vec<i8>>, Self::Error> {
        match self {
            TransmissionValue::VecI8(data) => Ok(data.into()),
            _ => Err(self),
        }
    }
}

impl From<VecDeque<Vec<i16>>> for TransmissionValue {
    fn from(value: VecDeque<Vec<i16>>) -> Self {
        TransmissionValue::VecI16(value)
    }
}

impl From<Vec<Vec<i16>>> for TransmissionValue {
    fn from(value: Vec<Vec<i16>>) -> Self {
        TransmissionValue::VecI16(value.into())
    }
}

impl TryInto<VecDeque<Vec<i16>>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<VecDeque<Vec<i16>>, Self::Error> {
        match self {
            TransmissionValue::VecI16(data) => Ok(data),
            _ => Err(self),
        }
    }
}

impl TryInto<Vec<Vec<i16>>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<Vec<Vec<i16>>, Self::Error> {
        match self {
            TransmissionValue::VecI16(data) => Ok(data.into()),
            _ => Err(self),
        }
    }
}

impl From<VecDeque<Vec<i32>>> for TransmissionValue {
    fn from(value: VecDeque<Vec<i32>>) -> Self {
        TransmissionValue::VecI32(value)
    }
}

impl From<Vec<Vec<i32>>> for TransmissionValue {
    fn from(value: Vec<Vec<i32>>) -> Self {
        TransmissionValue::VecI32(value.into())
    }
}

impl TryInto<VecDeque<Vec<i32>>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<VecDeque<Vec<i32>>, Self::Error> {
        match self {
            TransmissionValue::VecI32(data) => Ok(data),
            _ => Err(self),
        }
    }
}

impl TryInto<Vec<Vec<i32>>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<Vec<Vec<i32>>, Self::Error> {
        match self {
            TransmissionValue::VecI32(data) => Ok(data.into()),
            _ => Err(self),
        }
    }
}

impl From<VecDeque<Vec<i64>>> for TransmissionValue {
    fn from(value: VecDeque<Vec<i64>>) -> Self {
        TransmissionValue::VecI64(value)
    }
}

impl From<Vec<Vec<i64>>> for TransmissionValue {
    fn from(value: Vec<Vec<i64>>) -> Self {
        TransmissionValue::VecI64(value.into())
    }
}

impl TryInto<VecDeque<Vec<i64>>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<VecDeque<Vec<i64>>, Self::Error> {
        match self {
            TransmissionValue::VecI64(data) => Ok(data),
            _ => Err(self),
        }
    }
}

impl TryInto<Vec<Vec<i64>>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<Vec<Vec<i64>>, Self::Error> {
        match self {
            TransmissionValue::VecI64(data) => Ok(data.into()),
            _ => Err(self),
        }
    }
}

impl From<VecDeque<Vec<i128>>> for TransmissionValue {
    fn from(value: VecDeque<Vec<i128>>) -> Self {
        TransmissionValue::VecI128(value)
    }
}

impl From<Vec<Vec<i128>>> for TransmissionValue {
    fn from(value: Vec<Vec<i128>>) -> Self {
        TransmissionValue::VecI128(value.into())
    }
}

impl TryInto<VecDeque<Vec<i128>>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<VecDeque<Vec<i128>>, Self::Error> {
        match self {
            TransmissionValue::VecI128(data) => Ok(data),
            _ => Err(self),
        }
    }
}

impl TryInto<Vec<Vec<i128>>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<Vec<Vec<i128>>, Self::Error> {
        match self {
            TransmissionValue::VecI128(data) => Ok(data.into()),
            _ => Err(self),
        }
    }
}

impl From<VecDeque<Vec<u8>>> for TransmissionValue {
    fn from(value: VecDeque<Vec<u8>>) -> Self {
        TransmissionValue::VecU8(value)
    }
}

impl From<Vec<Vec<u8>>> for TransmissionValue {
    fn from(value: Vec<Vec<u8>>) -> Self {
        TransmissionValue::VecU8(value.into())
    }
}

impl TryInto<VecDeque<Vec<u8>>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<VecDeque<Vec<u8>>, Self::Error> {
        match self {
            TransmissionValue::VecU8(data) => Ok(data),
            TransmissionValue::VecByte(data) => Ok(data),
            _ => Err(self),
        }
    }
}

impl TryInto<Vec<Vec<u8>>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<Vec<Vec<u8>>, Self::Error> {
        match self {
            TransmissionValue::VecU8(data) => Ok(data.into()),
            TransmissionValue::VecByte(data) => Ok(data.into()),
            _ => Err(self),
        }
    }
}

impl From<VecDeque<Vec<u16>>> for TransmissionValue {
    fn from(value: VecDeque<Vec<u16>>) -> Self {
        TransmissionValue::VecU16(value)
    }
}

impl From<Vec<Vec<u16>>> for TransmissionValue {
    fn from(value: Vec<Vec<u16>>) -> Self {
        TransmissionValue::VecU16(value.into())
    }
}

impl TryInto<VecDeque<Vec<u16>>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<VecDeque<Vec<u16>>, Self::Error> {
        match self {
            TransmissionValue::VecU16(data) => Ok(data),
            _ => Err(self),
        }
    }
}

impl TryInto<Vec<Vec<u16>>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<Vec<Vec<u16>>, Self::Error> {
        match self {
            TransmissionValue::VecU16(data) => Ok(data.into()),
            _ => Err(self),
        }
    }
}

impl From<VecDeque<Vec<u32>>> for TransmissionValue {
    fn from(value: VecDeque<Vec<u32>>) -> Self {
        TransmissionValue::VecU32(value)
    }
}

impl From<Vec<Vec<u32>>> for TransmissionValue {
    fn from(value: Vec<Vec<u32>>) -> Self {
        TransmissionValue::VecU32(value.into())
    }
}

impl TryInto<VecDeque<Vec<u32>>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<VecDeque<Vec<u32>>, Self::Error> {
        match self {
            TransmissionValue::VecU32(data) => Ok(data),
            _ => Err(self),
        }
    }
}

impl TryInto<Vec<Vec<u32>>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<Vec<Vec<u32>>, Self::Error> {
        match self {
            TransmissionValue::VecU32(data) => Ok(data.into()),
            _ => Err(self),
        }
    }
}

impl From<VecDeque<Vec<u64>>> for TransmissionValue {
    fn from(value: VecDeque<Vec<u64>>) -> Self {
        TransmissionValue::VecU64(value)
    }
}

impl From<Vec<Vec<u64>>> for TransmissionValue {
    fn from(value: Vec<Vec<u64>>) -> Self {
        TransmissionValue::VecU64(value.into())
    }
}

impl TryInto<VecDeque<Vec<u64>>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<VecDeque<Vec<u64>>, Self::Error> {
        match self {
            TransmissionValue::VecU64(data) => Ok(data),
            _ => Err(self),
        }
    }
}

impl TryInto<Vec<Vec<u64>>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<Vec<Vec<u64>>, Self::Error> {
        match self {
            TransmissionValue::VecU64(data) => Ok(data.into()),
            _ => Err(self),
        }
    }
}

impl From<VecDeque<Vec<u128>>> for TransmissionValue {
    fn from(value: VecDeque<Vec<u128>>) -> Self {
        TransmissionValue::VecU128(value)
    }
}

impl From<Vec<Vec<u128>>> for TransmissionValue {
    fn from(value: Vec<Vec<u128>>) -> Self {
        TransmissionValue::VecU128(value.into())
    }
}

impl TryInto<VecDeque<Vec<u128>>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<VecDeque<Vec<u128>>, Self::Error> {
        match self {
            TransmissionValue::VecU128(data) => Ok(data),
            _ => Err(self),
        }
    }
}

impl TryInto<Vec<Vec<u128>>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<Vec<Vec<u128>>, Self::Error> {
        match self {
            TransmissionValue::VecU128(data) => Ok(data.into()),
            _ => Err(self),
        }
    }
}

impl From<VecDeque<Vec<f32>>> for TransmissionValue {
    fn from(value: VecDeque<Vec<f32>>) -> Self {
        TransmissionValue::VecF32(value)
    }
}

impl From<Vec<Vec<f32>>> for TransmissionValue {
    fn from(value: Vec<Vec<f32>>) -> Self {
        TransmissionValue::VecF32(value.into())
    }
}

impl TryInto<VecDeque<Vec<f32>>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<VecDeque<Vec<f32>>, Self::Error> {
        match self {
            TransmissionValue::VecF32(data) => Ok(data),
            _ => Err(self),
        }
    }
}

impl TryInto<Vec<Vec<f32>>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<Vec<Vec<f32>>, Self::Error> {
        match self {
            TransmissionValue::VecF32(data) => Ok(data.into()),
            _ => Err(self),
        }
    }
}

impl From<VecDeque<Vec<f64>>> for TransmissionValue {
    fn from(value: VecDeque<Vec<f64>>) -> Self {
        TransmissionValue::VecF64(value)
    }
}

impl From<Vec<Vec<f64>>> for TransmissionValue {
    fn from(value: Vec<Vec<f64>>) -> Self {
        TransmissionValue::VecF64(value.into())
    }
}

impl TryInto<VecDeque<Vec<f64>>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<VecDeque<Vec<f64>>, Self::Error> {
        match self {
            TransmissionValue::VecF64(data) => Ok(data),
            _ => Err(self),
        }
    }
}

impl TryInto<Vec<Vec<f64>>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<Vec<Vec<f64>>, Self::Error> {
        match self {
            TransmissionValue::VecF64(data) => Ok(data.into()),
            _ => Err(self),
        }
    }
}

impl From<VecDeque<Vec<bool>>> for TransmissionValue {
    fn from(value: VecDeque<Vec<bool>>) -> Self {
        TransmissionValue::VecBool(value)
    }
}

impl From<Vec<Vec<bool>>> for TransmissionValue {
    fn from(value: Vec<Vec<bool>>) -> Self {
        TransmissionValue::VecBool(value.into())
    }
}

impl TryInto<VecDeque<Vec<bool>>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<VecDeque<Vec<bool>>, Self::Error> {
        match self {
            TransmissionValue::VecBool(data) => Ok(data),
            _ => Err(self),
        }
    }
}

impl TryInto<Vec<Vec<bool>>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<Vec<Vec<bool>>, Self::Error> {
        match self {
            TransmissionValue::VecBool(data) => Ok(data.into()),
            _ => Err(self),
        }
    }
}

impl From<VecDeque<Vec<char>>> for TransmissionValue {
    fn from(value: VecDeque<Vec<char>>) -> Self {
        TransmissionValue::VecChar(value)
    }
}

impl From<Vec<Vec<char>>> for TransmissionValue {
    fn from(value: Vec<Vec<char>>) -> Self {
        TransmissionValue::VecChar(value.into())
    }
}

impl TryInto<VecDeque<Vec<char>>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<VecDeque<Vec<char>>, Self::Error> {
        match self {
            TransmissionValue::VecChar(data) => Ok(data),
            _ => Err(self),
        }
    }
}

impl TryInto<Vec<Vec<char>>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<Vec<Vec<char>>, Self::Error> {
        match self {
            TransmissionValue::VecChar(data) => Ok(data.into()),
            _ => Err(self),
        }
    }
}

impl From<VecDeque<Vec<String>>> for TransmissionValue {
    fn from(value: VecDeque<Vec<String>>) -> Self {
        TransmissionValue::VecString(value)
    }
}

impl From<Vec<Vec<String>>> for TransmissionValue {
    fn from(value: Vec<Vec<String>>) -> Self {
        TransmissionValue::VecString(value.into())
    }
}

impl TryInto<VecDeque<Vec<String>>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<VecDeque<Vec<String>>, Self::Error> {
        match self {
            TransmissionValue::VecString(data) => Ok(data),
            _ => Err(self),
        }
    }
}

impl TryInto<Vec<Vec<String>>> for TransmissionValue {
    type Error = Self;

    fn try_into(self) -> Result<Vec<Vec<String>>, Self::Error> {
        match self {
            TransmissionValue::VecString(data) => Ok(data.into()),
            _ => Err(self),
        }
    }
}
