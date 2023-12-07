use crate::descriptor::{DataType, Structure, Type};
use core::fmt::{self, Debug, Display, Formatter};
use std::convert::TryInto;

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

    VecVoid(Vec<()>),

    VecI8(Vec<i8>),
    VecI16(Vec<i16>),
    VecI32(Vec<i32>),
    VecI64(Vec<i64>),
    VecI128(Vec<i128>),

    VecU8(Vec<u8>),
    VecU16(Vec<u16>),
    VecU32(Vec<u32>),
    VecU64(Vec<u64>),
    VecU128(Vec<u128>),

    VecF32(Vec<f32>),
    VecF64(Vec<f64>),

    VecBool(Vec<bool>),
    VecByte(Vec<u8>),
    VecChar(Vec<char>),
    VecString(Vec<String>),
}

impl From<()> for Value {
    fn from(value: ()) -> Self {
        Value::Void(())
    }
}

impl TryInto<()> for Value {
    type Error = Self;

    fn try_into(self) -> Result<(), Self::Error> {
        match self {
            Value::Void(_) => Ok(()),
            _ => Err(self),
        }
    }
}

impl From<i8> for Value {
    fn from(value: i8) -> Self {
        Value::I8(value)
    }
}

impl TryInto<i8> for Value {
    type Error = Self;

    fn try_into(self) -> Result<i8, Self::Error> {
        match self {
            Value::I8(val) => Ok(val),
            _ => Err(self),
        }
    }
}
impl From<i16> for Value {
    fn from(value: i16) -> Self {
        Value::I16(value)
    }
}

impl TryInto<i16> for Value {
    type Error = Self;

    fn try_into(self) -> Result<i16, Self::Error> {
        match self {
            Value::I16(val) => Ok(val),
            _ => Err(self),
        }
    }
}
impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Value::I32(value)
    }
}

impl TryInto<i32> for Value {
    type Error = Self;

    fn try_into(self) -> Result<i32, Self::Error> {
        match self {
            Value::I32(val) => Ok(val),
            _ => Err(self),
        }
    }
}
impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Value::I64(value)
    }
}

impl TryInto<i64> for Value {
    type Error = Self;

    fn try_into(self) -> Result<i64, Self::Error> {
        match self {
            Value::I64(val) => Ok(val),
            _ => Err(self),
        }
    }
}
impl From<i128> for Value {
    fn from(value: i128) -> Self {
        Value::I128(value)
    }
}

impl TryInto<i128> for Value {
    type Error = Self;

    fn try_into(self) -> Result<i128, Self::Error> {
        match self {
            Value::I128(val) => Ok(val),
            _ => Err(self),
        }
    }
}

impl From<u8> for Value {
    fn from(value: u8) -> Self {
        Value::U8(value)
    }
}

impl TryInto<u8> for Value {
    type Error = Self;

    fn try_into(self) -> Result<u8, Self::Error> {
        match self {
            Value::U8(val) => Ok(val),
            Value::Byte(val) => Ok(val),
            _ => Err(self),
        }
    }
}
impl From<u16> for Value {
    fn from(value: u16) -> Self {
        Value::U16(value)
    }
}

impl TryInto<u16> for Value {
    type Error = Self;

    fn try_into(self) -> Result<u16, Self::Error> {
        match self {
            Value::U16(val) => Ok(val),
            _ => Err(self),
        }
    }
}
impl From<u32> for Value {
    fn from(value: u32) -> Self {
        Value::U32(value)
    }
}

impl TryInto<u32> for Value {
    type Error = Self;

    fn try_into(self) -> Result<u32, Self::Error> {
        match self {
            Value::U32(val) => Ok(val),
            _ => Err(self),
        }
    }
}
impl From<u64> for Value {
    fn from(value: u64) -> Self {
        Value::U64(value)
    }
}

impl TryInto<u64> for Value {
    type Error = Self;

    fn try_into(self) -> Result<u64, Self::Error> {
        match self {
            Value::U64(val) => Ok(val),
            _ => Err(self),
        }
    }
}
impl From<u128> for Value {
    fn from(value: u128) -> Self {
        Value::U128(value)
    }
}

impl TryInto<u128> for Value {
    type Error = Self;

    fn try_into(self) -> Result<u128, Self::Error> {
        match self {
            Value::U128(val) => Ok(val),
            _ => Err(self),
        }
    }
}

impl From<f32> for Value {
    fn from(value: f32) -> Self {
        Value::F32(value)
    }
}

impl TryInto<f32> for Value {
    type Error = Self;

    fn try_into(self) -> Result<f32, Self::Error> {
        match self {
            Value::F32(val) => Ok(val),
            _ => Err(self),
        }
    }
}
impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::F64(value)
    }
}

impl TryInto<f64> for Value {
    type Error = Self;

    fn try_into(self) -> Result<f64, Self::Error> {
        match self {
            Value::F64(val) => Ok(val),
            _ => Err(self),
        }
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Bool(value)
    }
}

impl TryInto<bool> for Value {
    type Error = Self;

    fn try_into(self) -> Result<bool, Self::Error> {
        match self {
            Value::Bool(val) => Ok(val),
            _ => Err(self),
        }
    }
}
impl From<char> for Value {
    fn from(value: char) -> Self {
        Value::Char(value)
    }
}

impl TryInto<char> for Value {
    type Error = Self;

    fn try_into(self) -> Result<char, Self::Error> {
        match self {
            Value::Char(val) => Ok(val),
            _ => Err(self),
        }
    }
}
impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::String(value)
    }
}

impl TryInto<String> for Value {
    type Error = Self;

    fn try_into(self) -> Result<String, Self::Error> {
        match self {
            Value::String(val) => Ok(val),
            _ => Err(self),
        }
    }
}

impl From<Vec<()>> for Value {
    fn from(value: Vec<()>) -> Self {
        Value::VecVoid(value)
    }
}

impl TryInto<Vec<()>> for Value {
    type Error = Self;

    fn try_into(self) -> Result<Vec<()>, Self::Error> {
        match self {
            Value::VecVoid(val) => Ok(val),
            _ => Err(self),
        }
    }
}

impl From<Vec<i8>> for Value {
    fn from(value: Vec<i8>) -> Self {
        Value::VecI8(value)
    }
}

impl TryInto<Vec<i8>> for Value {
    type Error = Self;

    fn try_into(self) -> Result<Vec<i8>, Self::Error> {
        match self {
            Value::VecI8(val) => Ok(val),
            _ => Err(self),
        }
    }
}
impl From<Vec<i16>> for Value {
    fn from(value: Vec<i16>) -> Self {
        Value::VecI16(value)
    }
}

impl TryInto<Vec<i16>> for Value {
    type Error = Self;

    fn try_into(self) -> Result<Vec<i16>, Self::Error> {
        match self {
            Value::VecI16(val) => Ok(val),
            _ => Err(self),
        }
    }
}
impl From<Vec<i32>> for Value {
    fn from(value: Vec<i32>) -> Self {
        Value::VecI32(value)
    }
}

impl TryInto<Vec<i32>> for Value {
    type Error = Self;

    fn try_into(self) -> Result<Vec<i32>, Self::Error> {
        match self {
            Value::VecI32(val) => Ok(val),
            _ => Err(self),
        }
    }
}
impl From<Vec<i64>> for Value {
    fn from(value: Vec<i64>) -> Self {
        Value::VecI64(value)
    }
}

impl TryInto<Vec<i64>> for Value {
    type Error = Self;

    fn try_into(self) -> Result<Vec<i64>, Self::Error> {
        match self {
            Value::VecI64(val) => Ok(val),
            _ => Err(self),
        }
    }
}
impl From<Vec<i128>> for Value {
    fn from(value: Vec<i128>) -> Self {
        Value::VecI128(value)
    }
}

impl TryInto<Vec<i128>> for Value {
    type Error = Self;

    fn try_into(self) -> Result<Vec<i128>, Self::Error> {
        match self {
            Value::VecI128(val) => Ok(val),
            _ => Err(self),
        }
    }
}

impl From<Vec<u8>> for Value {
    fn from(value: Vec<u8>) -> Self {
        Value::VecU8(value)
    }
}

impl TryInto<Vec<u8>> for Value {
    type Error = Self;

    fn try_into(self) -> Result<Vec<u8>, Self::Error> {
        match self {
            Value::VecU8(val) => Ok(val),
            Value::VecByte(val) => Ok(val),
            _ => Err(self),
        }
    }
}
impl From<Vec<u16>> for Value {
    fn from(value: Vec<u16>) -> Self {
        Value::VecU16(value)
    }
}

impl TryInto<Vec<u16>> for Value {
    type Error = Self;

    fn try_into(self) -> Result<Vec<u16>, Self::Error> {
        match self {
            Value::VecU16(val) => Ok(val),
            _ => Err(self),
        }
    }
}
impl From<Vec<u32>> for Value {
    fn from(value: Vec<u32>) -> Self {
        Value::VecU32(value)
    }
}

impl TryInto<Vec<u32>> for Value {
    type Error = Self;

    fn try_into(self) -> Result<Vec<u32>, Self::Error> {
        match self {
            Value::VecU32(val) => Ok(val),
            _ => Err(self),
        }
    }
}
impl From<Vec<u64>> for Value {
    fn from(value: Vec<u64>) -> Self {
        Value::VecU64(value)
    }
}

impl TryInto<Vec<u64>> for Value {
    type Error = Self;

    fn try_into(self) -> Result<Vec<u64>, Self::Error> {
        match self {
            Value::VecU64(val) => Ok(val),
            _ => Err(self),
        }
    }
}
impl From<Vec<u128>> for Value {
    fn from(value: Vec<u128>) -> Self {
        Value::VecU128(value)
    }
}

impl TryInto<Vec<u128>> for Value {
    type Error = Self;

    fn try_into(self) -> Result<Vec<u128>, Self::Error> {
        match self {
            Value::VecU128(val) => Ok(val),
            _ => Err(self),
        }
    }
}

impl From<Vec<f32>> for Value {
    fn from(value: Vec<f32>) -> Self {
        Value::VecF32(value)
    }
}

impl TryInto<Vec<f32>> for Value {
    type Error = Self;

    fn try_into(self) -> Result<Vec<f32>, Self::Error> {
        match self {
            Value::VecF32(val) => Ok(val),
            _ => Err(self),
        }
    }
}
impl From<Vec<f64>> for Value {
    fn from(value: Vec<f64>) -> Self {
        Value::VecF64(value)
    }
}

impl TryInto<Vec<f64>> for Value {
    type Error = Self;

    fn try_into(self) -> Result<Vec<f64>, Self::Error> {
        match self {
            Value::VecF64(val) => Ok(val),
            _ => Err(self),
        }
    }
}

impl From<Vec<bool>> for Value {
    fn from(value: Vec<bool>) -> Self {
        Value::VecBool(value)
    }
}

impl TryInto<Vec<bool>> for Value {
    type Error = Self;

    fn try_into(self) -> Result<Vec<bool>, Self::Error> {
        match self {
            Value::VecBool(val) => Ok(val),
            _ => Err(self),
        }
    }
}
impl From<Vec<char>> for Value {
    fn from(value: Vec<char>) -> Self {
        Value::VecChar(value)
    }
}

impl TryInto<Vec<char>> for Value {
    type Error = Self;

    fn try_into(self) -> Result<Vec<char>, Self::Error> {
        match self {
            Value::VecChar(val) => Ok(val),
            _ => Err(self),
        }
    }
}
impl From<Vec<String>> for Value {
    fn from(value: Vec<String>) -> Self {
        Value::VecString(value)
    }
}

impl TryInto<Vec<String>> for Value {
    type Error = Self;

    fn try_into(self) -> Result<Vec<String>, Self::Error> {
        match self {
            Value::VecString(val) => Ok(val),
            _ => Err(self),
        }
    }
}

impl Value {
    pub fn void(self) -> () {
        match self {
            Value::Void(v) => v,
            _ => panic!("void value expected"),
        }
    }

    pub fn u8(self) -> u8 {
        match self {
            Value::U8(v) => v,
            _ => panic!("u8 value expected"),
        }
    }

    pub fn u16(self) -> u16 {
        match self {
            Value::U16(v) => v,
            _ => panic!("u16 value expected"),
        }
    }

    pub fn u32(self) -> u32 {
        match self {
            Value::U32(v) => v,
            _ => panic!("u32 value expected"),
        }
    }

    pub fn u64(self) -> u64 {
        match self {
            Value::U64(v) => v,
            _ => panic!("u64 value expected"),
        }
    }

    pub fn u128(self) -> u128 {
        match self {
            Value::U128(v) => v,
            _ => panic!("u128 value expected"),
        }
    }

    pub fn i8(self) -> i8 {
        match self {
            Value::I8(v) => v,
            _ => panic!("i8 value expected"),
        }
    }

    pub fn i16(self) -> i16 {
        match self {
            Value::I16(v) => v,
            _ => panic!("i16 value expected"),
        }
    }

    pub fn i32(self) -> i32 {
        match self {
            Value::I32(v) => v,
            _ => panic!("i32 value expected"),
        }
    }

    pub fn i64(self) -> i64 {
        match self {
            Value::I64(v) => v,
            _ => panic!("i64 value expected"),
        }
    }

    pub fn i128(self) -> i128 {
        match self {
            Value::I128(v) => v,
            _ => panic!("i128 value expected"),
        }
    }

    pub fn f32(self) -> f32 {
        match self {
            Value::F32(v) => v,
            _ => panic!("f32 value expected"),
        }
    }

    pub fn f64(self) -> f64 {
        match self {
            Value::F64(v) => v,
            _ => panic!("f64 value expected"),
        }
    }

    pub fn bool(self) -> bool {
        match self {
            Value::Bool(v) => v,
            _ => panic!("bool value expected"),
        }
    }

    pub fn byte(self) -> u8 {
        match self {
            Value::Byte(v) => v,
            _ => panic!("byte value expected"),
        }
    }

    pub fn char(self) -> char {
        match self {
            Value::Char(v) => v,
            _ => panic!("char value expected"),
        }
    }

    pub fn string(self) -> String {
        match self {
            Value::String(v) => v,
            _ => panic!("string value expected"),
        }
    }

    pub fn vec_void(self) -> Vec<()> {
        match self {
            Value::VecVoid(v) => v,
            _ => panic!("Vec<void> value expected"),
        }
    }

    pub fn vec_u8(self) -> Vec<u8> {
        match self {
            Value::VecU8(v) => v,
            _ => panic!("Vec<u8> value expected"),
        }
    }

    pub fn vec_u16(self) -> Vec<u16> {
        match self {
            Value::VecU16(v) => v,
            _ => panic!("Vec<u16> value expected"),
        }
    }

    pub fn vec_u32(self) -> Vec<u32> {
        match self {
            Value::VecU32(v) => v,
            _ => panic!("Vec<u32> value expected"),
        }
    }

    pub fn vec_u64(self) -> Vec<u64> {
        match self {
            Value::VecU64(v) => v,
            _ => panic!("Vec<u64> value expected"),
        }
    }

    pub fn vec_u128(self) -> Vec<u128> {
        match self {
            Value::VecU128(v) => v,
            _ => panic!("Vec<u128> value expected"),
        }
    }

    pub fn vec_i8(self) -> Vec<i8> {
        match self {
            Value::VecI8(v) => v,
            _ => panic!("Vec<i8> value expected"),
        }
    }

    pub fn vec_i16(self) -> Vec<i16> {
        match self {
            Value::VecI16(v) => v,
            _ => panic!("Vec<i16> value expected"),
        }
    }

    pub fn vec_i32(self) -> Vec<i32> {
        match self {
            Value::VecI32(v) => v,
            _ => panic!("Vec<i32> value expected"),
        }
    }

    pub fn vec_i64(self) -> Vec<i64> {
        match self {
            Value::VecI64(v) => v,
            _ => panic!("Vec<i64> value expected"),
        }
    }

    pub fn vec_i128(self) -> Vec<i128> {
        match self {
            Value::VecI128(v) => v,
            _ => panic!("Vec<i128> value expected"),
        }
    }

    pub fn vec_f32(self) -> Vec<f32> {
        match self {
            Value::VecF32(v) => v,
            _ => panic!("Vec<f32> value expected"),
        }
    }

    pub fn vec_f64(self) -> Vec<f64> {
        match self {
            Value::VecF64(v) => v,
            _ => panic!("Vec<f64> value expected"),
        }
    }

    pub fn vec_bool(self) -> Vec<bool> {
        match self {
            Value::VecBool(v) => v,
            _ => panic!("Vec<bool> value expected"),
        }
    }

    pub fn vec_byte(self) -> Vec<u8> {
        match self {
            Value::VecByte(v) => v,
            _ => panic!("Vec<byte> value expected"),
        }
    }

    pub fn vec_char(self) -> Vec<char> {
        match self {
            Value::VecChar(v) => v,
            _ => panic!("Vec<char> value expected"),
        }
    }

    pub fn vec_string(self) -> Vec<String> {
        match self {
            Value::VecString(v) => v,
            _ => panic!("Vec<string> value expected"),
        }
    }

    pub fn datatype(&self) -> DataType {
        match self {
            Value::Void(_) => DataType::new(Structure::Scalar, Type::Void),

            Value::I8(_) => DataType::new(Structure::Scalar, Type::I8),
            Value::I16(_) => DataType::new(Structure::Scalar, Type::I16),
            Value::I32(_) => DataType::new(Structure::Scalar, Type::I32),
            Value::I64(_) => DataType::new(Structure::Scalar, Type::I64),
            Value::I128(_) => DataType::new(Structure::Scalar, Type::I128),

            Value::U8(_) => DataType::new(Structure::Scalar, Type::U8),
            Value::U16(_) => DataType::new(Structure::Scalar, Type::U16),
            Value::U32(_) => DataType::new(Structure::Scalar, Type::U32),
            Value::U64(_) => DataType::new(Structure::Scalar, Type::U64),
            Value::U128(_) => DataType::new(Structure::Scalar, Type::U128),

            Value::F32(_) => DataType::new(Structure::Scalar, Type::F32),
            Value::F64(_) => DataType::new(Structure::Scalar, Type::F64),

            Value::Bool(_) => DataType::new(Structure::Scalar, Type::Bool),
            Value::Byte(_) => DataType::new(Structure::Scalar, Type::Byte),
            Value::Char(_) => DataType::new(Structure::Scalar, Type::Char),
            Value::String(_) => DataType::new(Structure::Scalar, Type::String),

            Value::VecVoid(_) => DataType::new(Structure::Vector, Type::Void),

            Value::VecI8(_) => DataType::new(Structure::Vector, Type::I8),
            Value::VecI16(_) => DataType::new(Structure::Vector, Type::I16),
            Value::VecI32(_) => DataType::new(Structure::Vector, Type::I32),
            Value::VecI64(_) => DataType::new(Structure::Vector, Type::I64),
            Value::VecI128(_) => DataType::new(Structure::Vector, Type::I128),

            Value::VecU8(_) => DataType::new(Structure::Vector, Type::U8),
            Value::VecU16(_) => DataType::new(Structure::Vector, Type::U16),
            Value::VecU32(_) => DataType::new(Structure::Vector, Type::U32),
            Value::VecU64(_) => DataType::new(Structure::Vector, Type::U64),
            Value::VecU128(_) => DataType::new(Structure::Vector, Type::U128),

            Value::VecF32(_) => DataType::new(Structure::Vector, Type::F32),
            Value::VecF64(_) => DataType::new(Structure::Vector, Type::F64),

            Value::VecBool(_) => DataType::new(Structure::Vector, Type::Bool),
            Value::VecByte(_) => DataType::new(Structure::Vector, Type::Byte),
            Value::VecChar(_) => DataType::new(Structure::Vector, Type::Char),
            Value::VecString(_) => DataType::new(Structure::Vector, Type::String),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Value::Void(_) => write!(f, "()"),
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

            Value::VecVoid(v) => {
                let list: Vec<String> = v.iter().map(|_| format!("()")).collect();
                write!(f, "[{}]", list.join(", "))
            }

            Value::VecI8(v) => {
                let list: Vec<String> = v.iter().map(|v| format!("{}", v)).collect();
                write!(f, "[{}]", list.join(", "))
            }
            Value::VecI16(v) => {
                let list: Vec<String> = v.iter().map(|v| format!("{}", v)).collect();
                write!(f, "[{}]", list.join(", "))
            }
            Value::VecI32(v) => {
                let list: Vec<String> = v.iter().map(|v| format!("{}", v)).collect();
                write!(f, "[{}]", list.join(", "))
            }
            Value::VecI64(v) => {
                let list: Vec<String> = v.iter().map(|v| format!("{}", v)).collect();
                write!(f, "[{}]", list.join(", "))
            }
            Value::VecI128(v) => {
                let list: Vec<String> = v.iter().map(|v| format!("{}", v)).collect();
                write!(f, "[{}]", list.join(", "))
            }
            Value::VecU8(v) => {
                let list: Vec<String> = v.iter().map(|v| format!("{}", v)).collect();
                write!(f, "[{}]", list.join(", "))
            }
            Value::VecU16(v) => {
                let list: Vec<String> = v.iter().map(|v| format!("{}", v)).collect();
                write!(f, "[{}]", list.join(", "))
            }
            Value::VecU32(v) => {
                let list: Vec<String> = v.iter().map(|v| format!("{}", v)).collect();
                write!(f, "[{}]", list.join(", "))
            }
            Value::VecU64(v) => {
                let list: Vec<String> = v.iter().map(|v| format!("{}", v)).collect();
                write!(f, "[{}]", list.join(", "))
            }
            Value::VecU128(v) => {
                let list: Vec<String> = v.iter().map(|v| format!("{}", v)).collect();
                write!(f, "[{}]", list.join(", "))
            }
            Value::VecF32(v) => {
                let list: Vec<String> = v.iter().map(|v| format!("{}", v)).collect();
                write!(f, "[{}]", list.join(", "))
            }
            Value::VecF64(v) => {
                let list: Vec<String> = v.iter().map(|v| format!("{}", v)).collect();
                write!(f, "[{}]", list.join(", "))
            }
            Value::VecBool(v) => {
                let list: Vec<String> = v.iter().map(|v| format!("{}", v)).collect();
                write!(f, "[{}]", list.join(", "))
            }
            Value::VecByte(v) => {
                let list: Vec<String> = v.iter().map(|v| format!("{}", v)).collect();
                write!(f, "[{}]", list.join(", "))
            }
            Value::VecChar(v) => {
                let list: Vec<String> = v.iter().map(|v| format!("'{}'", v)).collect();
                write!(f, "[{}]", list.join(", "))
            }
            Value::VecString(v) => {
                let list: Vec<String> = v.iter().map(|v| format!("\"{}\"", v)).collect();
                write!(f, "[{}]", list.join(", "))
            }
        }
    }
}
