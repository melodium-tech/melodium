use crate::descriptor::DataType;
use core::fmt::{self, Debug, Display, Formatter};

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

/// Trait allowing to get real data based on Rust type.
///
/// This trait exist to circumvent E0119 that is disabling us to use TryInto.
/// See https://github.com/rust-lang/rust/issues/50133
pub trait GetData<T>: Sized {
    fn try_data(self) -> Result<T, ()>;
}

impl From<()> for Value {
    fn from(value: ()) -> Self {
        Value::Void(value)
    }
}

impl GetData<()> for Value {
    fn try_data(self) -> Result<(), ()> {
        match self {
            Value::Void(_) => Ok(()),
            _ => Err(()),
        }
    }
}

impl From<i8> for Value {
    fn from(value: i8) -> Self {
        Value::I8(value)
    }
}

impl GetData<i8> for Value {
    fn try_data(self) -> Result<i8, ()> {
        match self {
            Value::I8(val) => Ok(val),
            _ => Err(()),
        }
    }
}
impl From<i16> for Value {
    fn from(value: i16) -> Self {
        Value::I16(value)
    }
}

impl GetData<i16> for Value {
    fn try_data(self) -> Result<i16, ()> {
        match self {
            Value::I16(val) => Ok(val),
            _ => Err(()),
        }
    }
}
impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Value::I32(value)
    }
}

impl GetData<i32> for Value {
    fn try_data(self) -> Result<i32, ()> {
        match self {
            Value::I32(val) => Ok(val),
            _ => Err(()),
        }
    }
}
impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Value::I64(value)
    }
}

impl GetData<i64> for Value {
    fn try_data(self) -> Result<i64, ()> {
        match self {
            Value::I64(val) => Ok(val),
            _ => Err(()),
        }
    }
}
impl From<i128> for Value {
    fn from(value: i128) -> Self {
        Value::I128(value)
    }
}

impl GetData<i128> for Value {
    fn try_data(self) -> Result<i128, ()> {
        match self {
            Value::I128(val) => Ok(val),
            _ => Err(()),
        }
    }
}

impl From<u8> for Value {
    fn from(value: u8) -> Self {
        Value::U8(value)
    }
}

impl GetData<u8> for Value {
    fn try_data(self) -> Result<u8, ()> {
        match self {
            Value::U8(val) => Ok(val),
            Value::Byte(val) => Ok(val),
            _ => Err(()),
        }
    }
}
impl From<u16> for Value {
    fn from(value: u16) -> Self {
        Value::U16(value)
    }
}

impl GetData<u16> for Value {
    fn try_data(self) -> Result<u16, ()> {
        match self {
            Value::U16(val) => Ok(val),
            _ => Err(()),
        }
    }
}
impl From<u32> for Value {
    fn from(value: u32) -> Self {
        Value::U32(value)
    }
}

impl GetData<u32> for Value {
    fn try_data(self) -> Result<u32, ()> {
        match self {
            Value::U32(val) => Ok(val),
            _ => Err(()),
        }
    }
}
impl From<u64> for Value {
    fn from(value: u64) -> Self {
        Value::U64(value)
    }
}

impl GetData<u64> for Value {
    fn try_data(self) -> Result<u64, ()> {
        match self {
            Value::U64(val) => Ok(val),
            _ => Err(()),
        }
    }
}
impl From<u128> for Value {
    fn from(value: u128) -> Self {
        Value::U128(value)
    }
}

impl GetData<u128> for Value {
    fn try_data(self) -> Result<u128, ()> {
        match self {
            Value::U128(val) => Ok(val),
            _ => Err(()),
        }
    }
}

impl From<f32> for Value {
    fn from(value: f32) -> Self {
        Value::F32(value)
    }
}

impl GetData<f32> for Value {
    fn try_data(self) -> Result<f32, ()> {
        match self {
            Value::F32(val) => Ok(val),
            _ => Err(()),
        }
    }
}
impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::F64(value)
    }
}

impl GetData<f64> for Value {
    fn try_data(self) -> Result<f64, ()> {
        match self {
            Value::F64(val) => Ok(val),
            _ => Err(()),
        }
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Bool(value)
    }
}

impl GetData<bool> for Value {
    fn try_data(self) -> Result<bool, ()> {
        match self {
            Value::Bool(val) => Ok(val),
            _ => Err(()),
        }
    }
}
impl From<char> for Value {
    fn from(value: char) -> Self {
        Value::Char(value)
    }
}

impl GetData<char> for Value {
    fn try_data(self) -> Result<char, ()> {
        match self {
            Value::Char(val) => Ok(val),
            _ => Err(()),
        }
    }
}
impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::String(value)
    }
}

impl GetData<String> for Value {
    fn try_data(self) -> Result<String, ()> {
        match self {
            Value::String(val) => Ok(val),
            _ => Err(()),
        }
    }
}

impl<T: Into<Value>> From<Option<T>> for Value {
    fn from(value: Option<T>) -> Self {
        Value::Option(value.map(|val| Box::new(val.into())))
    }
}

impl<T> GetData<Option<T>> for Value
where
    Self: GetData<T>,
{
    fn try_data(self) -> Result<Option<T>, ()> {
        match self {
            Value::Option(val) => {
                if let Some(val) = val {
                    match val.try_data() {
                        Ok(val) => Ok(Some(val)),
                        Err(_) => Err(()),
                    }
                } else {
                    Ok(None)
                }
            }
            _ => Err(()),
        }
    }
}

impl<T: Into<Value>> From<Vec<T>> for Value {
    fn from(value: Vec<T>) -> Self {
        Value::Vec(value.into_iter().map(|val| val.into()).collect())
    }
}

impl<T> GetData<Vec<T>> for Value
where
    Self: GetData<T>,
{
    fn try_data(self) -> Result<Vec<T>, ()> {
        match self {
            Value::Vec(val) => {
                let mut result = Vec::with_capacity(val.len());
                for val in val {
                    match val.try_data() {
                        Ok(val) => result.push(val),
                        Err(_) => return Err(()),
                    }
                }
                Ok(result)
            }
            _ => Err(()),
        }
    }
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
                .map(|val| val.datatype())
                .unwrap_or(DataType::Undetermined),
            Value::Vec(val) => val
                .first()
                .map(|val| val.datatype())
                .unwrap_or(DataType::Undetermined),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Value::Void(_) => write!(f, "_"),
            Value::I8(v) => write!(f, "{}", v),
            Value::I16(v) => write!(f, "{}", v),
            Value::I32(v) => write!(f, "{}", v),
            Value::I64(v) => write!(f, "{}", v),
            Value::I128(v) => write!(f, "{}", v),
            Value::U8(v) => write!(f, "{}", v),
            Value::U16(v) => write!(f, "{}", v),
            Value::U32(v) => write!(f, "{}", v),
            Value::U64(v) => write!(f, "{}", v),
            Value::U128(v) => write!(f, "{}", v),
            Value::F32(v) => write!(f, "{}", v),
            Value::F64(v) => write!(f, "{}", v),
            Value::Bool(v) => write!(f, "{}", v),
            Value::Byte(v) => write!(f, "0x{}", hex::encode([*v])),
            Value::Char(v) => write!(f, "'{}'", v),
            Value::String(v) => write!(f, "\"{}\"", v.replace('"', "\\\"")),
            Value::Option(v) => {
                if let Some(v) = v {
                    write!(f, "{v}")
                } else {
                    write!(f, "_")
                }
            }
            Value::Vec(v) => write!(
                f,
                "[{}]",
                v.iter()
                    .map(|val| val.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }
}
