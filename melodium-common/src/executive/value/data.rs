use super::super::Data;
use super::Value;
use std::sync::Arc;

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

impl From<Arc<dyn Data>> for Value {
    fn from(value: Arc<dyn Data>) -> Self {
        Value::Data(value)
    }
}

impl GetData<Arc<dyn Data>> for Value {
    fn try_data(self) -> Result<Arc<dyn Data>, ()> {
        match self {
            Value::Data(val) => Ok(val),
            _ => Err(()),
        }
    }
}
