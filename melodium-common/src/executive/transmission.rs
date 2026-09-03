use super::{Data, PackedArray, Value};
use std::collections::VecDeque;
use std::convert::TryInto;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum TransmissionError {
    NoReceiver,
    EverythingClosed,
    NoData,
    /// A received value's runtime type didn't match the type it was cast to (see
    /// `InputExt::recv_one_as`/`recv_many_as`). Replaces what used to be an `.unwrap()`
    /// panic at call sites doing this cast by hand.
    TypeMismatch,
}

pub type SendResult = Result<(), TransmissionError>;
pub type RecvResult<T> = Result<T, TransmissionError>;

#[derive(Clone, Debug, PartialEq)]
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

    // The `Packed*` variants below are the batch-of-ticks counterpart of `Value::Packed`:
    // each queue entry is one whole packed array (one `Stream<Vec<T>>` tick, or a lone
    // `Vec<T>` value), not one scalar. They deliberately don't share storage with the
    // scalar variants above (`I8`, `U8`, ...) even for the same `T` — flattening a
    // `Stream<Vec<byte>>` into the same buffer as a `Stream<byte>` would destroy the
    // boundary between ticks (each array can be a different length). See ticket #116.
    PackedI8(VecDeque<Arc<Vec<i8>>>),
    PackedI16(VecDeque<Arc<Vec<i16>>>),
    PackedI32(VecDeque<Arc<Vec<i32>>>),
    PackedI64(VecDeque<Arc<Vec<i64>>>),
    PackedI128(VecDeque<Arc<Vec<i128>>>),

    PackedU8(VecDeque<Arc<Vec<u8>>>),
    PackedU16(VecDeque<Arc<Vec<u16>>>),
    PackedU32(VecDeque<Arc<Vec<u32>>>),
    PackedU64(VecDeque<Arc<Vec<u64>>>),
    PackedU128(VecDeque<Arc<Vec<u128>>>),

    PackedF32(VecDeque<Arc<Vec<f32>>>),
    PackedF64(VecDeque<Arc<Vec<f64>>>),

    PackedBool(VecDeque<Arc<Vec<bool>>>),
    PackedByte(VecDeque<Arc<Vec<u8>>>),
    PackedChar(VecDeque<Arc<Vec<char>>>),

    /// This variant handle all non-optimized cases.
    ///
    /// Optimized (and non-optimized) cases are at the implementation discretion.
    Other(VecDeque<Value>),
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

            Value::Packed(PackedArray::I8(value)) => TransmissionValue::PackedI8({
                let mut vec = VecDeque::new();
                vec.push_back(value);
                vec
            }),
            Value::Packed(PackedArray::I16(value)) => TransmissionValue::PackedI16({
                let mut vec = VecDeque::new();
                vec.push_back(value);
                vec
            }),
            Value::Packed(PackedArray::I32(value)) => TransmissionValue::PackedI32({
                let mut vec = VecDeque::new();
                vec.push_back(value);
                vec
            }),
            Value::Packed(PackedArray::I64(value)) => TransmissionValue::PackedI64({
                let mut vec = VecDeque::new();
                vec.push_back(value);
                vec
            }),
            Value::Packed(PackedArray::I128(value)) => TransmissionValue::PackedI128({
                let mut vec = VecDeque::new();
                vec.push_back(value);
                vec
            }),
            Value::Packed(PackedArray::U8(value)) => TransmissionValue::PackedU8({
                let mut vec = VecDeque::new();
                vec.push_back(value);
                vec
            }),
            Value::Packed(PackedArray::U16(value)) => TransmissionValue::PackedU16({
                let mut vec = VecDeque::new();
                vec.push_back(value);
                vec
            }),
            Value::Packed(PackedArray::U32(value)) => TransmissionValue::PackedU32({
                let mut vec = VecDeque::new();
                vec.push_back(value);
                vec
            }),
            Value::Packed(PackedArray::U64(value)) => TransmissionValue::PackedU64({
                let mut vec = VecDeque::new();
                vec.push_back(value);
                vec
            }),
            Value::Packed(PackedArray::U128(value)) => TransmissionValue::PackedU128({
                let mut vec = VecDeque::new();
                vec.push_back(value);
                vec
            }),
            Value::Packed(PackedArray::F32(value)) => TransmissionValue::PackedF32({
                let mut vec = VecDeque::new();
                vec.push_back(value);
                vec
            }),
            Value::Packed(PackedArray::F64(value)) => TransmissionValue::PackedF64({
                let mut vec = VecDeque::new();
                vec.push_back(value);
                vec
            }),
            Value::Packed(PackedArray::Bool(value)) => TransmissionValue::PackedBool({
                let mut vec = VecDeque::new();
                vec.push_back(value);
                vec
            }),
            Value::Packed(PackedArray::Byte(value)) => TransmissionValue::PackedByte({
                let mut vec = VecDeque::new();
                vec.push_back(value);
                vec
            }),
            Value::Packed(PackedArray::Char(value)) => TransmissionValue::PackedChar({
                let mut vec = VecDeque::new();
                vec.push_back(value);
                vec
            }),

            _ => TransmissionValue::Other({
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

            (TransmissionValue::PackedI8(data), TransmissionValue::PackedI8(mut values)) => {
                data.append(&mut values)
            }
            (TransmissionValue::PackedI16(data), TransmissionValue::PackedI16(mut values)) => {
                data.append(&mut values)
            }
            (TransmissionValue::PackedI32(data), TransmissionValue::PackedI32(mut values)) => {
                data.append(&mut values)
            }
            (TransmissionValue::PackedI64(data), TransmissionValue::PackedI64(mut values)) => {
                data.append(&mut values)
            }
            (TransmissionValue::PackedI128(data), TransmissionValue::PackedI128(mut values)) => {
                data.append(&mut values)
            }
            (TransmissionValue::PackedU8(data), TransmissionValue::PackedU8(mut values)) => {
                data.append(&mut values)
            }
            (TransmissionValue::PackedU16(data), TransmissionValue::PackedU16(mut values)) => {
                data.append(&mut values)
            }
            (TransmissionValue::PackedU32(data), TransmissionValue::PackedU32(mut values)) => {
                data.append(&mut values)
            }
            (TransmissionValue::PackedU64(data), TransmissionValue::PackedU64(mut values)) => {
                data.append(&mut values)
            }
            (TransmissionValue::PackedU128(data), TransmissionValue::PackedU128(mut values)) => {
                data.append(&mut values)
            }
            (TransmissionValue::PackedF32(data), TransmissionValue::PackedF32(mut values)) => {
                data.append(&mut values)
            }
            (TransmissionValue::PackedF64(data), TransmissionValue::PackedF64(mut values)) => {
                data.append(&mut values)
            }
            (TransmissionValue::PackedBool(data), TransmissionValue::PackedBool(mut values)) => {
                data.append(&mut values)
            }
            (TransmissionValue::PackedByte(data), TransmissionValue::PackedByte(mut values)) => {
                data.append(&mut values)
            }
            (TransmissionValue::PackedChar(data), TransmissionValue::PackedChar(mut values)) => {
                data.append(&mut values)
            }

            (TransmissionValue::Other(data), TransmissionValue::Other(mut values)) => {
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
            TransmissionValue::PackedI8(data) => data.len(),
            TransmissionValue::PackedI16(data) => data.len(),
            TransmissionValue::PackedI32(data) => data.len(),
            TransmissionValue::PackedI64(data) => data.len(),
            TransmissionValue::PackedI128(data) => data.len(),
            TransmissionValue::PackedU8(data) => data.len(),
            TransmissionValue::PackedU16(data) => data.len(),
            TransmissionValue::PackedU32(data) => data.len(),
            TransmissionValue::PackedU64(data) => data.len(),
            TransmissionValue::PackedU128(data) => data.len(),
            TransmissionValue::PackedF32(data) => data.len(),
            TransmissionValue::PackedF64(data) => data.len(),
            TransmissionValue::PackedBool(data) => data.len(),
            TransmissionValue::PackedByte(data) => data.len(),
            TransmissionValue::PackedChar(data) => data.len(),
            TransmissionValue::Other(data) => data.len(),
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
            TransmissionValue::PackedI8(data) => data
                .pop_front()
                .map(|data| Value::Packed(PackedArray::I8(data))),
            TransmissionValue::PackedI16(data) => data
                .pop_front()
                .map(|data| Value::Packed(PackedArray::I16(data))),
            TransmissionValue::PackedI32(data) => data
                .pop_front()
                .map(|data| Value::Packed(PackedArray::I32(data))),
            TransmissionValue::PackedI64(data) => data
                .pop_front()
                .map(|data| Value::Packed(PackedArray::I64(data))),
            TransmissionValue::PackedI128(data) => data
                .pop_front()
                .map(|data| Value::Packed(PackedArray::I128(data))),
            TransmissionValue::PackedU8(data) => data
                .pop_front()
                .map(|data| Value::Packed(PackedArray::U8(data))),
            TransmissionValue::PackedU16(data) => data
                .pop_front()
                .map(|data| Value::Packed(PackedArray::U16(data))),
            TransmissionValue::PackedU32(data) => data
                .pop_front()
                .map(|data| Value::Packed(PackedArray::U32(data))),
            TransmissionValue::PackedU64(data) => data
                .pop_front()
                .map(|data| Value::Packed(PackedArray::U64(data))),
            TransmissionValue::PackedU128(data) => data
                .pop_front()
                .map(|data| Value::Packed(PackedArray::U128(data))),
            TransmissionValue::PackedF32(data) => data
                .pop_front()
                .map(|data| Value::Packed(PackedArray::F32(data))),
            TransmissionValue::PackedF64(data) => data
                .pop_front()
                .map(|data| Value::Packed(PackedArray::F64(data))),
            TransmissionValue::PackedBool(data) => data
                .pop_front()
                .map(|data| Value::Packed(PackedArray::Bool(data))),
            TransmissionValue::PackedByte(data) => data
                .pop_front()
                .map(|data| Value::Packed(PackedArray::Byte(data))),
            TransmissionValue::PackedChar(data) => data
                .pop_front()
                .map(|data| Value::Packed(PackedArray::Char(data))),
            TransmissionValue::Other(data) => data.pop_front(),
        }
    }

    /// Rough memory footprint of the whole batch, in bytes. See `Value::estimated_size`
    /// for why this is an estimate rather than an exact figure. The optimized variants
    /// (fixed-size scalars, packed `Byte`) are O(1); `String` and `Other` are O(n) since
    /// their elements don't have a uniform size.
    pub fn estimated_size(&self) -> usize {
        match self {
            TransmissionValue::Void(data) => data.len() * std::mem::size_of::<()>(),
            TransmissionValue::I8(data) => data.len() * std::mem::size_of::<i8>(),
            TransmissionValue::I16(data) => data.len() * std::mem::size_of::<i16>(),
            TransmissionValue::I32(data) => data.len() * std::mem::size_of::<i32>(),
            TransmissionValue::I64(data) => data.len() * std::mem::size_of::<i64>(),
            TransmissionValue::I128(data) => data.len() * std::mem::size_of::<i128>(),
            TransmissionValue::U8(data) => data.len() * std::mem::size_of::<u8>(),
            TransmissionValue::U16(data) => data.len() * std::mem::size_of::<u16>(),
            TransmissionValue::U32(data) => data.len() * std::mem::size_of::<u32>(),
            TransmissionValue::U64(data) => data.len() * std::mem::size_of::<u64>(),
            TransmissionValue::U128(data) => data.len() * std::mem::size_of::<u128>(),
            TransmissionValue::F32(data) => data.len() * std::mem::size_of::<f32>(),
            TransmissionValue::F64(data) => data.len() * std::mem::size_of::<f64>(),
            TransmissionValue::Bool(data) => data.len() * std::mem::size_of::<bool>(),
            TransmissionValue::Byte(data) => data.len(),
            TransmissionValue::Char(data) => data.len() * std::mem::size_of::<char>(),
            TransmissionValue::String(data) => data.iter().map(String::len).sum(),
            TransmissionValue::PackedI8(data) => data
                .iter()
                .map(|arr| arr.len() * std::mem::size_of::<i8>())
                .sum(),
            TransmissionValue::PackedI16(data) => data
                .iter()
                .map(|arr| arr.len() * std::mem::size_of::<i16>())
                .sum(),
            TransmissionValue::PackedI32(data) => data
                .iter()
                .map(|arr| arr.len() * std::mem::size_of::<i32>())
                .sum(),
            TransmissionValue::PackedI64(data) => data
                .iter()
                .map(|arr| arr.len() * std::mem::size_of::<i64>())
                .sum(),
            TransmissionValue::PackedI128(data) => data
                .iter()
                .map(|arr| arr.len() * std::mem::size_of::<i128>())
                .sum(),
            TransmissionValue::PackedU8(data) => data
                .iter()
                .map(|arr| arr.len() * std::mem::size_of::<u8>())
                .sum(),
            TransmissionValue::PackedU16(data) => data
                .iter()
                .map(|arr| arr.len() * std::mem::size_of::<u16>())
                .sum(),
            TransmissionValue::PackedU32(data) => data
                .iter()
                .map(|arr| arr.len() * std::mem::size_of::<u32>())
                .sum(),
            TransmissionValue::PackedU64(data) => data
                .iter()
                .map(|arr| arr.len() * std::mem::size_of::<u64>())
                .sum(),
            TransmissionValue::PackedU128(data) => data
                .iter()
                .map(|arr| arr.len() * std::mem::size_of::<u128>())
                .sum(),
            TransmissionValue::PackedF32(data) => data
                .iter()
                .map(|arr| arr.len() * std::mem::size_of::<f32>())
                .sum(),
            TransmissionValue::PackedF64(data) => data
                .iter()
                .map(|arr| arr.len() * std::mem::size_of::<f64>())
                .sum(),
            TransmissionValue::PackedBool(data) => data
                .iter()
                .map(|arr| arr.len() * std::mem::size_of::<bool>())
                .sum(),
            TransmissionValue::PackedByte(data) => data.iter().map(|arr| arr.len()).sum(),
            TransmissionValue::PackedChar(data) => data
                .iter()
                .map(|arr| arr.len() * std::mem::size_of::<char>())
                .sum(),
            TransmissionValue::Other(data) => data.iter().map(Value::estimated_size).sum(),
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

            (TransmissionValue::PackedI8(data), Value::Packed(PackedArray::I8(value))) => {
                data.push_back(value)
            }
            (TransmissionValue::PackedI16(data), Value::Packed(PackedArray::I16(value))) => {
                data.push_back(value)
            }
            (TransmissionValue::PackedI32(data), Value::Packed(PackedArray::I32(value))) => {
                data.push_back(value)
            }
            (TransmissionValue::PackedI64(data), Value::Packed(PackedArray::I64(value))) => {
                data.push_back(value)
            }
            (TransmissionValue::PackedI128(data), Value::Packed(PackedArray::I128(value))) => {
                data.push_back(value)
            }
            (TransmissionValue::PackedU8(data), Value::Packed(PackedArray::U8(value))) => {
                data.push_back(value)
            }
            (TransmissionValue::PackedU16(data), Value::Packed(PackedArray::U16(value))) => {
                data.push_back(value)
            }
            (TransmissionValue::PackedU32(data), Value::Packed(PackedArray::U32(value))) => {
                data.push_back(value)
            }
            (TransmissionValue::PackedU64(data), Value::Packed(PackedArray::U64(value))) => {
                data.push_back(value)
            }
            (TransmissionValue::PackedU128(data), Value::Packed(PackedArray::U128(value))) => {
                data.push_back(value)
            }
            (TransmissionValue::PackedF32(data), Value::Packed(PackedArray::F32(value))) => {
                data.push_back(value)
            }
            (TransmissionValue::PackedF64(data), Value::Packed(PackedArray::F64(value))) => {
                data.push_back(value)
            }
            (TransmissionValue::PackedBool(data), Value::Packed(PackedArray::Bool(value))) => {
                data.push_back(value)
            }
            (TransmissionValue::PackedByte(data), Value::Packed(PackedArray::Byte(value))) => {
                data.push_back(value)
            }
            (TransmissionValue::PackedChar(data), Value::Packed(PackedArray::Char(value))) => {
                data.push_back(value)
            }

            (TransmissionValue::Other(data), value) => data.push_back(value),

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
            TransmissionValue::PackedI8(data) => data
                .into_iter()
                .map(|data| Value::Packed(PackedArray::I8(data)))
                .collect(),
            TransmissionValue::PackedI16(data) => data
                .into_iter()
                .map(|data| Value::Packed(PackedArray::I16(data)))
                .collect(),
            TransmissionValue::PackedI32(data) => data
                .into_iter()
                .map(|data| Value::Packed(PackedArray::I32(data)))
                .collect(),
            TransmissionValue::PackedI64(data) => data
                .into_iter()
                .map(|data| Value::Packed(PackedArray::I64(data)))
                .collect(),
            TransmissionValue::PackedI128(data) => data
                .into_iter()
                .map(|data| Value::Packed(PackedArray::I128(data)))
                .collect(),
            TransmissionValue::PackedU8(data) => data
                .into_iter()
                .map(|data| Value::Packed(PackedArray::U8(data)))
                .collect(),
            TransmissionValue::PackedU16(data) => data
                .into_iter()
                .map(|data| Value::Packed(PackedArray::U16(data)))
                .collect(),
            TransmissionValue::PackedU32(data) => data
                .into_iter()
                .map(|data| Value::Packed(PackedArray::U32(data)))
                .collect(),
            TransmissionValue::PackedU64(data) => data
                .into_iter()
                .map(|data| Value::Packed(PackedArray::U64(data)))
                .collect(),
            TransmissionValue::PackedU128(data) => data
                .into_iter()
                .map(|data| Value::Packed(PackedArray::U128(data)))
                .collect(),
            TransmissionValue::PackedF32(data) => data
                .into_iter()
                .map(|data| Value::Packed(PackedArray::F32(data)))
                .collect(),
            TransmissionValue::PackedF64(data) => data
                .into_iter()
                .map(|data| Value::Packed(PackedArray::F64(data)))
                .collect(),
            TransmissionValue::PackedBool(data) => data
                .into_iter()
                .map(|data| Value::Packed(PackedArray::Bool(data)))
                .collect(),
            TransmissionValue::PackedByte(data) => data
                .into_iter()
                .map(|data| Value::Packed(PackedArray::Byte(data)))
                .collect(),
            TransmissionValue::PackedChar(data) => data
                .into_iter()
                .map(|data| Value::Packed(PackedArray::Char(data)))
                .collect(),
            TransmissionValue::Other(data) => data,
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
            TransmissionValue::PackedI8(data) => data
                .into_iter()
                .map(|data| Value::Packed(PackedArray::I8(data)))
                .collect(),
            TransmissionValue::PackedI16(data) => data
                .into_iter()
                .map(|data| Value::Packed(PackedArray::I16(data)))
                .collect(),
            TransmissionValue::PackedI32(data) => data
                .into_iter()
                .map(|data| Value::Packed(PackedArray::I32(data)))
                .collect(),
            TransmissionValue::PackedI64(data) => data
                .into_iter()
                .map(|data| Value::Packed(PackedArray::I64(data)))
                .collect(),
            TransmissionValue::PackedI128(data) => data
                .into_iter()
                .map(|data| Value::Packed(PackedArray::I128(data)))
                .collect(),
            TransmissionValue::PackedU8(data) => data
                .into_iter()
                .map(|data| Value::Packed(PackedArray::U8(data)))
                .collect(),
            TransmissionValue::PackedU16(data) => data
                .into_iter()
                .map(|data| Value::Packed(PackedArray::U16(data)))
                .collect(),
            TransmissionValue::PackedU32(data) => data
                .into_iter()
                .map(|data| Value::Packed(PackedArray::U32(data)))
                .collect(),
            TransmissionValue::PackedU64(data) => data
                .into_iter()
                .map(|data| Value::Packed(PackedArray::U64(data)))
                .collect(),
            TransmissionValue::PackedU128(data) => data
                .into_iter()
                .map(|data| Value::Packed(PackedArray::U128(data)))
                .collect(),
            TransmissionValue::PackedF32(data) => data
                .into_iter()
                .map(|data| Value::Packed(PackedArray::F32(data)))
                .collect(),
            TransmissionValue::PackedF64(data) => data
                .into_iter()
                .map(|data| Value::Packed(PackedArray::F64(data)))
                .collect(),
            TransmissionValue::PackedBool(data) => data
                .into_iter()
                .map(|data| Value::Packed(PackedArray::Bool(data)))
                .collect(),
            TransmissionValue::PackedByte(data) => data
                .into_iter()
                .map(|data| Value::Packed(PackedArray::Byte(data)))
                .collect(),
            TransmissionValue::PackedChar(data) => data
                .into_iter()
                .map(|data| Value::Packed(PackedArray::Char(data)))
                .collect(),
            TransmissionValue::Other(data) => data.into(),
        }
    }
}

// The conversions below (`From<VecDeque<T>>`, `From<Vec<T>>`, `TryInto<VecDeque<T>>`,
// `TryInto<Vec<T>>`) follow the exact same shape for every scalar type that maps 1:1 to a
// `TransmissionValue` variant, so they're generated rather than hand-duplicated per type —
// see ticket #120. `U8` is the one exception kept hand-written below: it's not a clean 1:1
// mapping, since `TryInto<Vec<u8>>`/`TryInto<VecDeque<u8>>` must also accept the `Byte`
// variant (bytes and small unsigned integers are interchangeable on extraction), while
// `Byte` itself has no corresponding `From` — it's only ever produced via `Value::Byte`
// going through `TransmissionValue::new`/`push`, not through a top-level `From<Vec<u8>>`.
macro_rules! transmission_scalar_type {
    ($variant:ident, $ty:ty) => {
        impl From<VecDeque<$ty>> for TransmissionValue {
            fn from(value: VecDeque<$ty>) -> Self {
                TransmissionValue::$variant(value)
            }
        }

        impl From<Vec<$ty>> for TransmissionValue {
            fn from(value: Vec<$ty>) -> Self {
                TransmissionValue::$variant(value.into())
            }
        }

        impl TryInto<VecDeque<$ty>> for TransmissionValue {
            type Error = ();

            fn try_into(self) -> Result<VecDeque<$ty>, Self::Error> {
                match self {
                    TransmissionValue::$variant(data) => Ok(data),
                    TransmissionValue::Other(data) => {
                        let mut vec = VecDeque::with_capacity(data.len());
                        for val in data {
                            if let Ok(val) = val.try_data() {
                                vec.push_back(val);
                            } else {
                                return Err(());
                            }
                        }
                        Ok(vec)
                    }
                    _ => Err(()),
                }
            }
        }

        impl TryInto<Vec<$ty>> for TransmissionValue {
            type Error = ();

            fn try_into(self) -> Result<Vec<$ty>, Self::Error> {
                match self {
                    TransmissionValue::$variant(data) => Ok(data.into()),
                    TransmissionValue::Other(data) => {
                        let mut vec = Vec::with_capacity(data.len());
                        for val in data {
                            if let Ok(val) = val.try_data() {
                                vec.push(val);
                            } else {
                                return Err(());
                            }
                        }
                        Ok(vec)
                    }
                    _ => Err(()),
                }
            }
        }
    };
}

transmission_scalar_type!(Void, ());
transmission_scalar_type!(I8, i8);
transmission_scalar_type!(I16, i16);
transmission_scalar_type!(I32, i32);
transmission_scalar_type!(I64, i64);
transmission_scalar_type!(I128, i128);
transmission_scalar_type!(U16, u16);
transmission_scalar_type!(U32, u32);
transmission_scalar_type!(U64, u64);
transmission_scalar_type!(U128, u128);
transmission_scalar_type!(F32, f32);
transmission_scalar_type!(F64, f64);
transmission_scalar_type!(Bool, bool);
transmission_scalar_type!(Char, char);
transmission_scalar_type!(String, String);

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
    type Error = ();

    fn try_into(self) -> Result<VecDeque<u8>, Self::Error> {
        match self {
            TransmissionValue::U8(data) => Ok(data),
            TransmissionValue::Byte(data) => Ok(data),
            TransmissionValue::Other(data) => {
                let mut vec = VecDeque::with_capacity(data.len());
                for val in data {
                    if let Ok(val) = val.try_data() {
                        vec.push_back(val);
                    } else {
                        return Err(());
                    }
                }
                Ok(vec)
            }
            _ => Err(()),
        }
    }
}

impl TryInto<Vec<u8>> for TransmissionValue {
    type Error = ();

    fn try_into(self) -> Result<Vec<u8>, Self::Error> {
        match self {
            TransmissionValue::U8(data) => Ok(data.into()),
            TransmissionValue::Byte(data) => Ok(data.into()),
            TransmissionValue::Other(data) => {
                let mut vec = Vec::with_capacity(data.len());
                for val in data {
                    if let Ok(val) = val.try_data() {
                        vec.push(val);
                    } else {
                        return Err(());
                    }
                }
                Ok(vec)
            }
            _ => Err(()),
        }
    }
}

// `Arc<Vec<T>>` counterpart of `transmission_scalar_type!` above, for the `Packed*`
// variants: same generated shape (`From<VecDeque<_>>`, `From<Vec<_>>`, `TryInto<VecDeque<_>>`,
// `TryInto<Vec<_>>`), one array per queue entry instead of one scalar. `U8`/`Byte` are
// kept hand-written below for the same reason as their scalar counterparts: `From`
// must stay unambiguous (always produces `PackedU8`), while extraction accepts both.
macro_rules! transmission_packed_scalar_type {
    ($variant:ident, $ty:ty) => {
        impl From<VecDeque<Arc<Vec<$ty>>>> for TransmissionValue {
            fn from(value: VecDeque<Arc<Vec<$ty>>>) -> Self {
                TransmissionValue::$variant(value)
            }
        }

        impl From<Vec<Arc<Vec<$ty>>>> for TransmissionValue {
            fn from(value: Vec<Arc<Vec<$ty>>>) -> Self {
                TransmissionValue::$variant(value.into())
            }
        }

        impl TryInto<VecDeque<Arc<Vec<$ty>>>> for TransmissionValue {
            type Error = ();

            fn try_into(self) -> Result<VecDeque<Arc<Vec<$ty>>>, Self::Error> {
                match self {
                    TransmissionValue::$variant(data) => Ok(data),
                    TransmissionValue::Other(data) => {
                        let mut vec = VecDeque::with_capacity(data.len());
                        for val in data {
                            if let Ok(val) = val.try_data() {
                                vec.push_back(val);
                            } else {
                                return Err(());
                            }
                        }
                        Ok(vec)
                    }
                    _ => Err(()),
                }
            }
        }

        impl TryInto<Vec<Arc<Vec<$ty>>>> for TransmissionValue {
            type Error = ();

            fn try_into(self) -> Result<Vec<Arc<Vec<$ty>>>, Self::Error> {
                match self {
                    TransmissionValue::$variant(data) => Ok(data.into()),
                    TransmissionValue::Other(data) => {
                        let mut vec = Vec::with_capacity(data.len());
                        for val in data {
                            if let Ok(val) = val.try_data() {
                                vec.push(val);
                            } else {
                                return Err(());
                            }
                        }
                        Ok(vec)
                    }
                    _ => Err(()),
                }
            }
        }
    };
}

transmission_packed_scalar_type!(PackedI8, i8);
transmission_packed_scalar_type!(PackedI16, i16);
transmission_packed_scalar_type!(PackedI32, i32);
transmission_packed_scalar_type!(PackedI64, i64);
transmission_packed_scalar_type!(PackedI128, i128);
transmission_packed_scalar_type!(PackedU16, u16);
transmission_packed_scalar_type!(PackedU32, u32);
transmission_packed_scalar_type!(PackedU64, u64);
transmission_packed_scalar_type!(PackedU128, u128);
transmission_packed_scalar_type!(PackedF32, f32);
transmission_packed_scalar_type!(PackedF64, f64);
transmission_packed_scalar_type!(PackedBool, bool);
transmission_packed_scalar_type!(PackedChar, char);

impl From<VecDeque<Arc<Vec<u8>>>> for TransmissionValue {
    fn from(value: VecDeque<Arc<Vec<u8>>>) -> Self {
        TransmissionValue::PackedU8(value)
    }
}

impl From<Vec<Arc<Vec<u8>>>> for TransmissionValue {
    fn from(value: Vec<Arc<Vec<u8>>>) -> Self {
        TransmissionValue::PackedU8(value.into())
    }
}

impl TryInto<VecDeque<Arc<Vec<u8>>>> for TransmissionValue {
    type Error = ();

    fn try_into(self) -> Result<VecDeque<Arc<Vec<u8>>>, Self::Error> {
        match self {
            TransmissionValue::PackedU8(data) => Ok(data),
            TransmissionValue::PackedByte(data) => Ok(data),
            TransmissionValue::Other(data) => {
                let mut vec = VecDeque::with_capacity(data.len());
                for val in data {
                    if let Ok(val) = val.try_data() {
                        vec.push_back(val);
                    } else {
                        return Err(());
                    }
                }
                Ok(vec)
            }
            _ => Err(()),
        }
    }
}

impl TryInto<Vec<Arc<Vec<u8>>>> for TransmissionValue {
    type Error = ();

    fn try_into(self) -> Result<Vec<Arc<Vec<u8>>>, Self::Error> {
        match self {
            TransmissionValue::PackedU8(data) => Ok(data.into()),
            TransmissionValue::PackedByte(data) => Ok(data.into()),
            TransmissionValue::Other(data) => {
                let mut vec = Vec::with_capacity(data.len());
                for val in data {
                    if let Ok(val) = val.try_data() {
                        vec.push(val);
                    } else {
                        return Err(());
                    }
                }
                Ok(vec)
            }
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod conversion_tests {
    use super::*;

    // A macro-generated type, as a roundtrip sanity check on the generator itself.
    #[test]
    fn macro_generated_type_roundtrips() {
        let batch: TransmissionValue = vec![1i64, 2, 3].into();
        assert!(matches!(batch, TransmissionValue::I64(_)));
        let back: Vec<i64> = batch.try_into().unwrap();
        assert_eq!(back, vec![1, 2, 3]);
    }

    // Values boxed as `Other` (e.g. after arriving through a non-optimized path) must
    // still extract correctly through the generated `TryInto`, via `Value::try_data`.
    #[test]
    fn macro_generated_type_extracts_from_other_variant() {
        let boxed = TransmissionValue::Other(VecDeque::from(vec![Value::I64(10), Value::I64(20)]));
        let extracted: Vec<i64> = boxed.try_into().unwrap();
        assert_eq!(extracted, vec![10, 20]);
    }

    // Void's `()` payload exercises the macro too, not just numeric/string types.
    #[test]
    fn void_roundtrips_through_the_macro() {
        let batch: TransmissionValue = vec![(), (), ()].into();
        let back: Vec<()> = batch.try_into().unwrap();
        assert_eq!(back, vec![(), (), ()]);
    }

    // `Packed*` variants, one array per tick — a macro-generated type as a roundtrip
    // sanity check on `transmission_packed_scalar_type!`.
    #[test]
    fn packed_macro_generated_type_roundtrips_preserving_tick_boundaries() {
        let tick_one: Arc<Vec<i64>> = Arc::from(vec![1, 2, 3]);
        let tick_two: Arc<Vec<i64>> = Arc::from(vec![4, 5]);
        let batch: TransmissionValue = vec![Arc::clone(&tick_one), Arc::clone(&tick_two)].into();
        assert!(matches!(batch, TransmissionValue::PackedI64(_)));
        assert_eq!(batch.len(), 2);
        let back: Vec<Arc<Vec<i64>>> = batch.try_into().unwrap();
        assert_eq!(back, vec![tick_one, tick_two]);
    }

    // `push`/`pop_front`/`new` round-trip through `Value::Packed`, exactly as a real
    // `Output`/`Input` channel would use them.
    #[test]
    fn packed_value_roundtrips_through_new_push_and_pop_front() {
        let mut batch =
            TransmissionValue::new(Value::Packed(PackedArray::Byte(Arc::from(vec![1u8, 2, 3]))));
        batch.push(Value::Packed(PackedArray::Byte(Arc::from(vec![4u8, 5]))));
        assert_eq!(batch.len(), 2);
        assert_eq!(
            batch.pop_front(),
            Some(Value::Packed(PackedArray::Byte(Arc::from(vec![1u8, 2, 3]))))
        );
        assert_eq!(
            batch.pop_front(),
            Some(Value::Packed(PackedArray::Byte(Arc::from(vec![4u8, 5]))))
        );
        assert_eq!(batch.pop_front(), None);
    }

    // This is the packed counterpart of `u8_extraction_accepts_both_u8_and_byte_variants`
    // below: extraction must accept both `PackedU8` and `PackedByte`, even though only
    // `PackedU8` is ever produced by `From`.
    #[test]
    fn packed_u8_extraction_accepts_both_packed_u8_and_packed_byte_variants() {
        let as_u8 = TransmissionValue::PackedU8(VecDeque::from(vec![Arc::from(vec![1u8, 2, 3])]));
        let as_byte =
            TransmissionValue::PackedByte(VecDeque::from(vec![Arc::from(vec![4u8, 5, 6])]));

        let from_u8: Vec<Arc<Vec<u8>>> = as_u8.try_into().unwrap();
        let from_byte: Vec<Arc<Vec<u8>>> = as_byte.try_into().unwrap();

        assert_eq!(from_u8, vec![Arc::from(vec![1u8, 2, 3])]);
        assert_eq!(from_byte, vec![Arc::from(vec![4u8, 5, 6])]);
    }

    #[test]
    fn packed_construction_always_produces_packed_u8_not_packed_byte() {
        let batch: TransmissionValue = vec![Arc::<Vec<u8>>::from(vec![1u8, 2, 3])].into();
        assert!(matches!(batch, TransmissionValue::PackedU8(_)));
    }

    // This is the one case the ticket kept hand-written rather than folding into the
    // macro: extraction must accept both `U8` and `Byte`, even though only `U8` is ever
    // produced by `From`. Losing either arm here would silently break byte-stream reads.
    #[test]
    fn u8_extraction_accepts_both_u8_and_byte_variants() {
        let as_u8 = TransmissionValue::U8(VecDeque::from(vec![1u8, 2, 3]));
        let as_byte = TransmissionValue::Byte(VecDeque::from(vec![4u8, 5, 6]));

        let from_u8: Vec<u8> = as_u8.try_into().unwrap();
        let from_byte: Vec<u8> = as_byte.try_into().unwrap();

        assert_eq!(from_u8, vec![1, 2, 3]);
        assert_eq!(from_byte, vec![4, 5, 6]);
    }

    // From<Vec<u8>> must still only ever produce U8, never Byte — that asymmetry is
    // exactly why U8 couldn't go through the generic macro.
    #[test]
    fn u8_construction_always_produces_u8_variant_not_byte() {
        let batch: TransmissionValue = vec![1u8, 2, 3].into();
        assert!(matches!(batch, TransmissionValue::U8(_)));
    }

    #[test]
    fn mismatched_type_extraction_fails() {
        let batch = TransmissionValue::I64(VecDeque::from(vec![1]));
        let result: Result<Vec<String>, ()> = batch.try_into();
        assert!(result.is_err());
    }
}

impl From<VecDeque<Arc<dyn Data>>> for TransmissionValue {
    fn from(value: VecDeque<Arc<dyn Data>>) -> Self {
        TransmissionValue::Other(value.into_iter().map(|value| value.into()).collect())
    }
}

impl From<Vec<Arc<dyn Data>>> for TransmissionValue {
    fn from(value: Vec<Arc<dyn Data>>) -> Self {
        TransmissionValue::Other(value.into_iter().map(|value| value.into()).collect())
    }
}

impl TryInto<VecDeque<Arc<dyn Data>>> for TransmissionValue {
    type Error = ();

    fn try_into(self) -> Result<VecDeque<Arc<dyn Data>>, Self::Error> {
        match self {
            TransmissionValue::Other(data) => {
                let mut vec = VecDeque::with_capacity(data.len());
                for val in data {
                    if let Ok(val) = val.try_data() {
                        vec.push_back(val);
                    } else {
                        return Err(());
                    }
                }
                Ok(vec)
            }
            _ => Err(()),
        }
    }
}

impl TryInto<Vec<Arc<dyn Data>>> for TransmissionValue {
    type Error = ();

    fn try_into(self) -> Result<Vec<Arc<dyn Data>>, Self::Error> {
        match self {
            TransmissionValue::Other(data) => {
                let mut vec = Vec::with_capacity(data.len());
                for val in data {
                    if let Ok(val) = val.try_data() {
                        vec.push(val);
                    } else {
                        return Err(());
                    }
                }
                Ok(vec)
            }
            _ => Err(()),
        }
    }
}

/// Generic counterpart to the `Arc<dyn Data>` impls above, for a concrete `Data`
/// implementor `D` — mirrors `GetData<Arc<D>> for Value` (see `value/data.rs`) at the
/// batch level, so `recv_many_as::<Arc<D>>()` works the same way `recv_one_as::<Arc<D>>()`
/// already does. Coexists with the `Arc<dyn Data>` impls above for the same reason
/// `GetData<Arc<D>>` coexists with `GetData<Arc<dyn Data>>`: `D` carries an implicit
/// `Sized` bound here, which the unsized `dyn Data` can never satisfy.
impl<D: Data> From<VecDeque<Arc<D>>> for TransmissionValue {
    fn from(value: VecDeque<Arc<D>>) -> Self {
        TransmissionValue::Other(
            value
                .into_iter()
                .map(|value| (value as Arc<dyn Data>).into())
                .collect(),
        )
    }
}

impl<D: Data> From<Vec<Arc<D>>> for TransmissionValue {
    fn from(value: Vec<Arc<D>>) -> Self {
        TransmissionValue::Other(
            value
                .into_iter()
                .map(|value| (value as Arc<dyn Data>).into())
                .collect(),
        )
    }
}

impl<D: Data> TryInto<VecDeque<Arc<D>>> for TransmissionValue {
    type Error = ();

    fn try_into(self) -> Result<VecDeque<Arc<D>>, Self::Error> {
        match self {
            TransmissionValue::Other(data) => {
                let mut vec = VecDeque::with_capacity(data.len());
                for val in data {
                    if let Ok(val) = val.try_data::<Arc<D>>() {
                        vec.push_back(val);
                    } else {
                        return Err(());
                    }
                }
                Ok(vec)
            }
            _ => Err(()),
        }
    }
}

impl<D: Data> TryInto<Vec<Arc<D>>> for TransmissionValue {
    type Error = ();

    fn try_into(self) -> Result<Vec<Arc<D>>, Self::Error> {
        match self {
            TransmissionValue::Other(data) => {
                let mut vec = Vec::with_capacity(data.len());
                for val in data {
                    if let Ok(val) = val.try_data::<Arc<D>>() {
                        vec.push(val);
                    } else {
                        return Err(());
                    }
                }
                Ok(vec)
            }
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod estimated_size_tests {
    use super::*;

    #[test]
    fn packed_byte_variant_costs_one_byte_per_element() {
        let batch = TransmissionValue::Byte(VecDeque::from(vec![0u8; 4096]));
        assert_eq!(batch.estimated_size(), 4096);
    }

    #[test]
    fn fixed_size_scalar_variant_scales_with_type_size() {
        let batch = TransmissionValue::U64(VecDeque::from(vec![0u64; 10]));
        assert_eq!(batch.estimated_size(), 10 * std::mem::size_of::<u64>());
    }

    #[test]
    fn string_variant_sums_actual_string_lengths() {
        let batch =
            TransmissionValue::String(VecDeque::from(vec!["ab".to_string(), "cde".to_string()]));
        assert_eq!(batch.estimated_size(), 2 + 3);
    }

    #[test]
    fn other_variant_delegates_to_value_estimated_size() {
        let batch = TransmissionValue::Other(VecDeque::from(vec![Value::Byte(1), Value::Byte(2)]));
        assert_eq!(batch.estimated_size(), 2 * std::mem::size_of::<Value>());
    }

    #[test]
    fn packed_byte_variant_sums_every_arrays_length_with_no_per_element_overhead() {
        let batch = TransmissionValue::PackedByte(VecDeque::from(vec![
            Arc::from(vec![0u8; 4096]),
            Arc::from(vec![0u8; 10]),
        ]));
        assert_eq!(batch.estimated_size(), 4096 + 10);
    }

    #[test]
    fn packed_fixed_size_scalar_variant_scales_with_type_size() {
        let batch = TransmissionValue::PackedF64(VecDeque::from(vec![Arc::from(vec![0f64; 10])]));
        assert_eq!(batch.estimated_size(), 10 * std::mem::size_of::<f64>());
    }

    #[test]
    fn packed_variant_len_counts_ticks_not_elements() {
        // `len()` here means "how many arrays are queued", the same "one entry per
        // stream tick" meaning it has for the scalar variants - not the total element
        // count across every array, which is a different, tick-boundary-preserving
        // shape than a flat scalar `VecDeque<u8>` would have.
        let batch = TransmissionValue::PackedByte(VecDeque::from(vec![
            Arc::from(vec![1u8, 2, 3]),
            Arc::from(vec![4u8, 5]),
        ]));
        assert_eq!(batch.len(), 2);
    }
}
