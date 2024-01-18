use super::{GetData, Value};
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

#[derive(Clone, Debug)]
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
            TransmissionValue::Other(data) => data.pop_front(),
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
            TransmissionValue::Other(data) => data.into(),
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
    type Error = ();

    fn try_into(self) -> Result<VecDeque<()>, Self::Error> {
        match self {
            TransmissionValue::Void(data) => Ok(data),
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

impl TryInto<Vec<()>> for TransmissionValue {
    type Error = ();

    fn try_into(self) -> Result<Vec<()>, Self::Error> {
        match self {
            TransmissionValue::Void(data) => Ok(data.into()),
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
    type Error = ();

    fn try_into(self) -> Result<VecDeque<i8>, Self::Error> {
        match self {
            TransmissionValue::I8(data) => Ok(data),
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

impl TryInto<Vec<i8>> for TransmissionValue {
    type Error = ();

    fn try_into(self) -> Result<Vec<i8>, Self::Error> {
        match self {
            TransmissionValue::I8(data) => Ok(data.into()),
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
    type Error = ();

    fn try_into(self) -> Result<VecDeque<i16>, Self::Error> {
        match self {
            TransmissionValue::I16(data) => Ok(data),
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

impl TryInto<Vec<i16>> for TransmissionValue {
    type Error = ();

    fn try_into(self) -> Result<Vec<i16>, Self::Error> {
        match self {
            TransmissionValue::I16(data) => Ok(data.into()),
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
    type Error = ();

    fn try_into(self) -> Result<VecDeque<i32>, Self::Error> {
        match self {
            TransmissionValue::I32(data) => Ok(data),
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

impl TryInto<Vec<i32>> for TransmissionValue {
    type Error = ();

    fn try_into(self) -> Result<Vec<i32>, Self::Error> {
        match self {
            TransmissionValue::I32(data) => Ok(data.into()),
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
    type Error = ();

    fn try_into(self) -> Result<VecDeque<i64>, Self::Error> {
        match self {
            TransmissionValue::I64(data) => Ok(data),
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

impl TryInto<Vec<i64>> for TransmissionValue {
    type Error = ();

    fn try_into(self) -> Result<Vec<i64>, Self::Error> {
        match self {
            TransmissionValue::I64(data) => Ok(data.into()),
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
    type Error = ();

    fn try_into(self) -> Result<VecDeque<i128>, Self::Error> {
        match self {
            TransmissionValue::I128(data) => Ok(data),
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

impl TryInto<Vec<i128>> for TransmissionValue {
    type Error = ();

    fn try_into(self) -> Result<Vec<i128>, Self::Error> {
        match self {
            TransmissionValue::I128(data) => Ok(data.into()),
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
    type Error = ();

    fn try_into(self) -> Result<VecDeque<u16>, Self::Error> {
        match self {
            TransmissionValue::U16(data) => Ok(data),
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

impl TryInto<Vec<u16>> for TransmissionValue {
    type Error = ();

    fn try_into(self) -> Result<Vec<u16>, Self::Error> {
        match self {
            TransmissionValue::U16(data) => Ok(data.into()),
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
    type Error = ();

    fn try_into(self) -> Result<VecDeque<u32>, Self::Error> {
        match self {
            TransmissionValue::U32(data) => Ok(data),
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

impl TryInto<Vec<u32>> for TransmissionValue {
    type Error = ();

    fn try_into(self) -> Result<Vec<u32>, Self::Error> {
        match self {
            TransmissionValue::U32(data) => Ok(data.into()),
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
    type Error = ();

    fn try_into(self) -> Result<VecDeque<u64>, Self::Error> {
        match self {
            TransmissionValue::U64(data) => Ok(data),
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

impl TryInto<Vec<u64>> for TransmissionValue {
    type Error = ();

    fn try_into(self) -> Result<Vec<u64>, Self::Error> {
        match self {
            TransmissionValue::U64(data) => Ok(data.into()),
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
    type Error = ();

    fn try_into(self) -> Result<VecDeque<u128>, Self::Error> {
        match self {
            TransmissionValue::U128(data) => Ok(data),
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

impl TryInto<Vec<u128>> for TransmissionValue {
    type Error = ();

    fn try_into(self) -> Result<Vec<u128>, Self::Error> {
        match self {
            TransmissionValue::U128(data) => Ok(data.into()),
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
    type Error = ();

    fn try_into(self) -> Result<VecDeque<f32>, Self::Error> {
        match self {
            TransmissionValue::F32(data) => Ok(data),
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

impl TryInto<Vec<f32>> for TransmissionValue {
    type Error = ();

    fn try_into(self) -> Result<Vec<f32>, Self::Error> {
        match self {
            TransmissionValue::F32(data) => Ok(data.into()),
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
    type Error = ();

    fn try_into(self) -> Result<VecDeque<f64>, Self::Error> {
        match self {
            TransmissionValue::F64(data) => Ok(data),
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

impl TryInto<Vec<f64>> for TransmissionValue {
    type Error = ();

    fn try_into(self) -> Result<Vec<f64>, Self::Error> {
        match self {
            TransmissionValue::F64(data) => Ok(data.into()),
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
    type Error = ();

    fn try_into(self) -> Result<VecDeque<bool>, Self::Error> {
        match self {
            TransmissionValue::Bool(data) => Ok(data),
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

impl TryInto<Vec<bool>> for TransmissionValue {
    type Error = ();

    fn try_into(self) -> Result<Vec<bool>, Self::Error> {
        match self {
            TransmissionValue::Bool(data) => Ok(data.into()),
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
    type Error = ();

    fn try_into(self) -> Result<VecDeque<char>, Self::Error> {
        match self {
            TransmissionValue::Char(data) => Ok(data),
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

impl TryInto<Vec<char>> for TransmissionValue {
    type Error = ();

    fn try_into(self) -> Result<Vec<char>, Self::Error> {
        match self {
            TransmissionValue::Char(data) => Ok(data.into()),
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
    type Error = ();

    fn try_into(self) -> Result<VecDeque<String>, Self::Error> {
        match self {
            TransmissionValue::String(data) => Ok(data),
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

impl TryInto<Vec<String>> for TransmissionValue {
    type Error = ();

    fn try_into(self) -> Result<Vec<String>, Self::Error> {
        match self {
            TransmissionValue::String(data) => Ok(data.into()),
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
