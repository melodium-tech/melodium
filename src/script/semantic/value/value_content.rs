
use std::convert::TryFrom;
use crate::executive::value::Value as ExecutiveValue;
use crate::logic::descriptor::datatype::{DataType, Structure, Type};

use super::super::common::Reference;
use super::super::declared_parameter::DeclaredParameter;
use super::super::requirement::Requirement;

/// Enum holding value or reference designating the value.
pub enum ValueContent {
    Boolean(bool),
    Unsigned(u128),
    Integer(i128),
    Real(f64),
    String(String),
    /// Array, allowing recursive values (in case of vectors).
    Array(Vec<ValueContent>),
    /// Named value, referring to a parameter of the hosting sequence.
    Name(Reference<DeclaredParameter>),
    /// Context reference, referring to a requirement of the hosting sequence, and an inner element.
    ContextReference((Reference<Requirement>, String))
}

impl ValueContent {

    pub fn make_executive_value(&self, datatype: &DataType) -> Result<ExecutiveValue, String> {

        match datatype.structure() {
            Structure::Scalar => {
                match datatype.r#type() {

                    Type::U8 =>
                        match &self {
                            ValueContent::Unsigned(u) => {
                                if let Ok(u) = u8::try_from(*u) {
                                    Ok(ExecutiveValue::U8(u))
                                }
                                else {
                                    Err("Value too large for u8.".to_string())
                                }
                            },
                            ValueContent::Integer(i) => {
                                if let Ok(u) = u8::try_from(*i) {
                                    Ok(ExecutiveValue::U8(u))
                                }
                                else {
                                    Err("Value too large for u8.".to_string())
                                }
                            },
                            _ => Err("u8 value expected.".to_string())
                        },
                    
                    Type::U16 =>
                        match &self {
                            ValueContent::Unsigned(u) => {
                                if let Ok(u) = u16::try_from(*u) {
                                    Ok(ExecutiveValue::U16(u))
                                }
                                else {
                                    Err("Value too large for u16.".to_string())
                                }
                            },
                            ValueContent::Integer(i) => {
                                if let Ok(u) = u16::try_from(*i) {
                                    Ok(ExecutiveValue::U16(u))
                                }
                                else {
                                    Err("Value too large for u16.".to_string())
                                }
                            },
                            _ => Err("u16 value expected.".to_string())
                        },
                    
                    Type::U32 =>
                        match &self {
                            ValueContent::Unsigned(u) => {
                                if let Ok(u) = u32::try_from(*u) {
                                    Ok(ExecutiveValue::U32(u))
                                }
                                else {
                                    Err("Value too large for u32.".to_string())
                                }
                            },
                            ValueContent::Integer(i) => {
                                if let Ok(u) = u32::try_from(*i) {
                                    Ok(ExecutiveValue::U32(u))
                                }
                                else {
                                    Err("Value too large for u32.".to_string())
                                }
                            },
                            _ => Err("u32 value expected.".to_string())
                        },
                    
                    Type::U64 =>
                        match &self {
                            ValueContent::Unsigned(u) => {
                                if let Ok(u) = u64::try_from(*u) {
                                    Ok(ExecutiveValue::U64(u))
                                }
                                else {
                                    Err("Value too large for u64.".to_string())
                                }
                            },
                            ValueContent::Integer(i) => {
                                if let Ok(u) = u64::try_from(*i) {
                                    Ok(ExecutiveValue::U64(u))
                                }
                                else {
                                    Err("Value too large for u64.".to_string())
                                }
                            },
                            _ => Err("u64 value expected.".to_string())
                        },
                    
                    Type::U128 =>
                        match &self {
                            ValueContent::Unsigned(u) => Ok(ExecutiveValue::U128(*u)),
                            ValueContent::Integer(i) => {
                                if let Ok(u) = u128::try_from(*i) {
                                    Ok(ExecutiveValue::U128(u))
                                }
                                else {
                                    Err("Value too large for u128.".to_string())
                                }
                            },
                            _ => Err("u128 value expected.".to_string())
                        },

                    Type::I8 =>
                        match &self {
                            ValueContent::Unsigned(u) => {
                                if let Ok(i) = i8::try_from(*u) {
                                    Ok(ExecutiveValue::I8(i))
                                }
                                else {
                                    Err("Value too large for i8.".to_string())
                                }
                            },
                            ValueContent::Integer(i) => {
                                if let Ok(i) = i8::try_from(*i) {
                                    Ok(ExecutiveValue::I8(i))
                                }
                                else {
                                    Err("Value too large for i8.".to_string())
                                }
                            },
                            _ => Err("i8 value expected.".to_string())
                        },

                    Type::I16 =>
                        match &self {
                            ValueContent::Unsigned(u) => {
                                if let Ok(i) = i16::try_from(*u) {
                                    Ok(ExecutiveValue::I16(i))
                                }
                                else {
                                    Err("Value too large for i16.".to_string())
                                }
                            },
                            ValueContent::Integer(i) => {
                                if let Ok(i) = i16::try_from(*i) {
                                    Ok(ExecutiveValue::I16(i))
                                }
                                else {
                                    Err("Value too large for i16.".to_string())
                                }
                            },
                            _ => Err("i16 value expected.".to_string())
                        },
                    
                    Type::I32 =>
                        match &self {
                            ValueContent::Unsigned(u) => {
                                if let Ok(i) = i32::try_from(*u) {
                                    Ok(ExecutiveValue::I32(i))
                                }
                                else {
                                    Err("Value too large for i32.".to_string())
                                }
                            },
                            ValueContent::Integer(i) => {
                                if let Ok(i) = i32::try_from(*i) {
                                    Ok(ExecutiveValue::I32(i))
                                }
                                else {
                                    Err("Value too large for i32.".to_string())
                                }
                            },
                            _ => Err("i32 value expected.".to_string())
                        },

                    Type::I64 =>
                        match &self {
                            ValueContent::Unsigned(u) => {
                                if let Ok(i) = i64::try_from(*u) {
                                    Ok(ExecutiveValue::I64(i))
                                }
                                else {
                                    Err("Value too large for i64.".to_string())
                                }
                            },
                            ValueContent::Integer(i) => {
                                if let Ok(i) = i64::try_from(*i) {
                                    Ok(ExecutiveValue::I64(i))
                                }
                                else {
                                    Err("Value too large for i64.".to_string())
                                }
                            },
                            _ => Err("i64 value expected.".to_string())
                        },
                    
                    Type::I128 =>
                        match &self {
                            ValueContent::Unsigned(u) => {
                                if let Ok(i) = i128::try_from(*u) {
                                    Ok(ExecutiveValue::I128(i))
                                }
                                else {
                                    Err("Value too large for i128.".to_string())
                                }
                            },
                            ValueContent::Integer(i) => Ok(ExecutiveValue::I128(*i)),
                            _ => Err("i128 value expected.".to_string())
                        },
                    
                    Type::F32 =>
                        match &self {
                            ValueContent::Unsigned(u) => {
                                // We convert from u16 because f32 cannot hold greater values.
                                if let Ok(u) = u16::try_from(*u) {
                                    Ok(ExecutiveValue::F32(f32::from(u)))
                                }
                                else {
                                    Err("Value too large for f32.".to_string())
                                }
                            },
                            ValueContent::Integer(i) => {
                                // We convert from i16 because f32 cannot hold greater values.
                                if let Ok(i) = i16::try_from(*i) {
                                    Ok(ExecutiveValue::F32(f32::from(i)))
                                }
                                else {
                                    Err("Value too large for f32.".to_string())
                                }
                            },
                            ValueContent::Real(b) => Ok(ExecutiveValue::F32(*b as f32)),
                            _ => Err("f32 value expected.".to_string())
                        },
                    
                    Type::F64 =>
                        match &self {
                            ValueContent::Unsigned(u) => {
                                // We convert from u32 because f64 cannot hold greater values.
                                if let Ok(u) = u32::try_from(*u) {
                                    Ok(ExecutiveValue::F64(f64::from(u)))
                                }
                                else {
                                    Err("Value too large for f64.".to_string())
                                }
                            },
                            ValueContent::Integer(i) => {
                                // We convert from i32 because f64 cannot hold greater values.
                                if let Ok(i) = i32::try_from(*i) {
                                    Ok(ExecutiveValue::F64(f64::from(i)))
                                }
                                else {
                                    Err("Value too large for f64.".to_string())
                                }
                            },
                            ValueContent::Real(b) => Ok(ExecutiveValue::F64(*b)),
                            _ => Err("f64 value expected.".to_string())
                        },
                    
                    Type::Bool =>
                        match &self {
                            ValueContent::Boolean(b) => Ok(ExecutiveValue::Bool(*b)),
                            _ => Err("Boolean value expected.".to_string())
                        },
                    
                    Type::Byte =>
                        match &self {
                            ValueContent::Unsigned(u) => {
                                if let Ok(u) = u8::try_from(*u) {
                                    Ok(ExecutiveValue::Byte(u))
                                }
                                else {
                                    Err("Value too large for byte.".to_string())
                                }
                            },
                            ValueContent::Integer(i) => {
                                if let Ok(u) = u8::try_from(*i) {
                                    Ok(ExecutiveValue::Byte(u))
                                }
                                else {
                                    Err("Value too large for byte.".to_string())
                                }
                            },
                            _ => Err("byte value expected.".to_string())
                        },
                    
                    Type::Char =>
                        match &self {
                            ValueContent::Unsigned(u) => {
                                if let Ok(u) = u32::try_from(*u) {
                                    if let Some(c) = char::from_u32(u) {
                                        Ok(ExecutiveValue::Char(c))
                                    }
                                    else {
                                        Err("Value cannot be a char.".to_string())
                                    }
                                }
                                else {
                                    Err("Value too large for char.".to_string())
                                }
                            },
                            ValueContent::Integer(i) => {
                                if let Ok(u) = u32::try_from(*i) {
                                    if let Some(c) = char::from_u32(u) {
                                        Ok(ExecutiveValue::Char(c))
                                    }
                                    else {
                                        Err("Value cannot be a char.".to_string())
                                    }
                                }
                                else {
                                    Err("Value too large for char.".to_string())
                                }
                            },
                            _ => Err("char value expected.".to_string())
                        },

                    Type::String => 
                        match &self {
                            ValueContent::String(s) => Ok(ExecutiveValue::String(s.clone())),
                            _ => Err("String value expected.".to_string())
                        },

                }
            },
            Structure::Vector => {
                match datatype.r#type() {
                    Type::U8     => Ok(ExecutiveValue::VecU8(self.to_vector_u8()?)),
                    Type::U16    => Ok(ExecutiveValue::VecU16(self.to_vector_u16()?)),
                    Type::U32    => Ok(ExecutiveValue::VecU32(self.to_vector_u32()?)),
                    Type::U64    => Ok(ExecutiveValue::VecU64(self.to_vector_u64()?)),
                    Type::U128   => Ok(ExecutiveValue::VecU128(self.to_vector_u128()?)),
                    Type::I8     => Ok(ExecutiveValue::VecI8(self.to_vector_i8()?)),
                    Type::I16    => Ok(ExecutiveValue::VecI16(self.to_vector_i16()?)),
                    Type::I32    => Ok(ExecutiveValue::VecI32(self.to_vector_i32()?)),
                    Type::I64    => Ok(ExecutiveValue::VecI64(self.to_vector_i64()?)),
                    Type::I128   => Ok(ExecutiveValue::VecI128(self.to_vector_i128()?)),
                    Type::F32    => Ok(ExecutiveValue::VecF32(self.to_vector_f32()?)),
                    Type::F64    => Ok(ExecutiveValue::VecF64(self.to_vector_f64()?)),
                    Type::Bool   => Ok(ExecutiveValue::VecBool(self.to_vector_bool()?)),
                    Type::Byte   => Ok(ExecutiveValue::VecByte(self.to_vector_byte()?)),
                    Type::Char   => Ok(ExecutiveValue::VecChar(self.to_vector_char()?)),
                    Type::String => Ok(ExecutiveValue::VecString(self.to_vector_string()?)),
                }
            },
        }
    }

    pub fn to_vector_u8(&self) -> Result<Vec<u8>, String> {

        let datatype = DataType::new(Structure::Scalar, Type::U8);

        match self {
            ValueContent::Array(vec) => {
                let mut arr: Vec<u8> = Vec::with_capacity(vec.len());
                for val in vec {
                    match val.make_executive_value(&datatype)? {
                        ExecutiveValue::U8(u) => arr.push(u),
                        _ => panic!("Impossible semantic error case"),
                    }
                }
                Ok(arr)
            },
            _ => Err("Array of u8 values expected.".to_string())
        }
    }

    pub fn to_vector_u16(&self) -> Result<Vec<u16>, String> {

        let datatype = DataType::new(Structure::Scalar, Type::U16);

        match self {
            ValueContent::Array(vec) => {
                let mut arr: Vec<u16> = Vec::with_capacity(vec.len());
                for val in vec {
                    match val.make_executive_value(&datatype)? {
                        ExecutiveValue::U16(u) => arr.push(u),
                        _ => panic!("Impossible semantic error case"),
                    }
                }
                Ok(arr)
            },
            _ => Err("Array of u16 values expected.".to_string())
        }
    }

    pub fn to_vector_u32(&self) -> Result<Vec<u32>, String> {

        let datatype = DataType::new(Structure::Scalar, Type::U32);

        match self {
            ValueContent::Array(vec) => {
                let mut arr: Vec<u32> = Vec::with_capacity(vec.len());
                for val in vec {
                    match val.make_executive_value(&datatype)? {
                        ExecutiveValue::U32(u) => arr.push(u),
                        _ => panic!("Impossible semantic error case"),
                    }
                }
                Ok(arr)
            },
            _ => Err("Array of u32 values expected.".to_string())
        }
    }

    pub fn to_vector_u64(&self) -> Result<Vec<u64>, String> {

        let datatype = DataType::new(Structure::Scalar, Type::U64);

        match self {
            ValueContent::Array(vec) => {
                let mut arr: Vec<u64> = Vec::with_capacity(vec.len());
                for val in vec {
                    match val.make_executive_value(&datatype)? {
                        ExecutiveValue::U64(u) => arr.push(u),
                        _ => panic!("Impossible semantic error case"),
                    }
                }
                Ok(arr)
            },
            _ => Err("Array of u64 values expected.".to_string())
        }
    }

    pub fn to_vector_u128(&self) -> Result<Vec<u128>, String> {

        let datatype = DataType::new(Structure::Scalar, Type::U128);

        match self {
            ValueContent::Array(vec) => {
                let mut arr: Vec<u128> = Vec::with_capacity(vec.len());
                for val in vec {
                    match val.make_executive_value(&datatype)? {
                        ExecutiveValue::U128(u) => arr.push(u),
                        _ => panic!("Impossible semantic error case"),
                    }
                }
                Ok(arr)
            },
            _ => Err("Array of u128 values expected.".to_string())
        }
    }

    pub fn to_vector_i8(&self) -> Result<Vec<i8>, String> {

        let datatype = DataType::new(Structure::Scalar, Type::I8);

        match self {
            ValueContent::Array(vec) => {
                let mut arr: Vec<i8> = Vec::with_capacity(vec.len());
                for val in vec {
                    match val.make_executive_value(&datatype)? {
                        ExecutiveValue::I8(i) => arr.push(i),
                        _ => panic!("Impossible semantic error case"),
                    }
                }
                Ok(arr)
            },
            _ => Err("Array of i8 values expected.".to_string())
        }
    }

    pub fn to_vector_i16(&self) -> Result<Vec<i16>, String> {

        let datatype = DataType::new(Structure::Scalar, Type::I16);

        match self {
            ValueContent::Array(vec) => {
                let mut arr: Vec<i16> = Vec::with_capacity(vec.len());
                for val in vec {
                    match val.make_executive_value(&datatype)? {
                        ExecutiveValue::I16(i) => arr.push(i),
                        _ => panic!("Impossible semantic error case"),
                    }
                }
                Ok(arr)
            },
            _ => Err("Array of i16 values expected.".to_string())
        }
    }

    pub fn to_vector_i32(&self) -> Result<Vec<i32>, String> {

        let datatype = DataType::new(Structure::Scalar, Type::I32);

        match self {
            ValueContent::Array(vec) => {
                let mut arr: Vec<i32> = Vec::with_capacity(vec.len());
                for val in vec {
                    match val.make_executive_value(&datatype)? {
                        ExecutiveValue::I32(i) => arr.push(i),
                        _ => panic!("Impossible semantic error case"),
                    }
                }
                Ok(arr)
            },
            _ => Err("Array of i32 values expected.".to_string())
        }
    }

    pub fn to_vector_i64(&self) -> Result<Vec<i64>, String> {

        let datatype = DataType::new(Structure::Scalar, Type::I64);

        match self {
            ValueContent::Array(vec) => {
                let mut arr: Vec<i64> = Vec::with_capacity(vec.len());
                for val in vec {
                    match val.make_executive_value(&datatype)? {
                        ExecutiveValue::I64(i) => arr.push(i),
                        _ => panic!("Impossible semantic error case"),
                    }
                }
                Ok(arr)
            },
            _ => Err("Array of i64 values expected.".to_string())
        }
    }

    pub fn to_vector_i128(&self) -> Result<Vec<i128>, String> {

        let datatype = DataType::new(Structure::Scalar, Type::I128);

        match self {
            ValueContent::Array(vec) => {
                let mut arr: Vec<i128> = Vec::with_capacity(vec.len());
                for val in vec {
                    match val.make_executive_value(&datatype)? {
                        ExecutiveValue::I128(i) => arr.push(i),
                        _ => panic!("Impossible semantic error case"),
                    }
                }
                Ok(arr)
            },
            _ => Err("Array of i128 values expected.".to_string())
        }
    }

    pub fn to_vector_f32(&self) -> Result<Vec<f32>, String> {

        let datatype = DataType::new(Structure::Scalar, Type::F32);

        match self {
            ValueContent::Array(vec) => {
                let mut arr: Vec<f32> = Vec::with_capacity(vec.len());
                for val in vec {
                    match val.make_executive_value(&datatype)? {
                        ExecutiveValue::F32(f) => arr.push(f),
                        _ => panic!("Impossible semantic error case"),
                    }
                }
                Ok(arr)
            },
            _ => Err("Array of f32 values expected.".to_string())
        }
    }

    pub fn to_vector_f64(&self) -> Result<Vec<f64>, String> {

        let datatype = DataType::new(Structure::Scalar, Type::F64);

        match self {
            ValueContent::Array(vec) => {
                let mut arr: Vec<f64> = Vec::with_capacity(vec.len());
                for val in vec {
                    match val.make_executive_value(&datatype)? {
                        ExecutiveValue::F64(f) => arr.push(f),
                        _ => panic!("Impossible semantic error case"),
                    }
                }
                Ok(arr)
            },
            _ => Err("Array of f64 values expected.".to_string())
        }
    }

    pub fn to_vector_bool(&self) -> Result<Vec<bool>, String> {

        let datatype = DataType::new(Structure::Scalar, Type::Bool);

        match self {
            ValueContent::Array(vec) => {
                let mut arr: Vec<bool> = Vec::with_capacity(vec.len());
                for val in vec {
                    match val.make_executive_value(&datatype)? {
                        ExecutiveValue::Bool(b) => arr.push(b),
                        _ => panic!("Impossible semantic error case"),
                    }
                }
                Ok(arr)
            },
            _ => Err("Array of bool values expected.".to_string())
        }
    }

    pub fn to_vector_byte(&self) -> Result<Vec<u8>, String> {

        let datatype = DataType::new(Structure::Scalar, Type::Byte);

        match self {
            ValueContent::Array(vec) => {
                let mut arr: Vec<u8> = Vec::with_capacity(vec.len());
                for val in vec {
                    match val.make_executive_value(&datatype)? {
                        ExecutiveValue::Byte(b) => arr.push(b),
                        _ => panic!("Impossible semantic error case"),
                    }
                }
                Ok(arr)
            },
            _ => Err("Array of byte values expected.".to_string())
        }
    }

    pub fn to_vector_char(&self) -> Result<Vec<char>, String> {

        let datatype = DataType::new(Structure::Scalar, Type::Char);

        match self {
            ValueContent::Array(vec) => {
                let mut arr: Vec<char> = Vec::with_capacity(vec.len());
                for val in vec {
                    match val.make_executive_value(&datatype)? {
                        ExecutiveValue::Char(c) => arr.push(c),
                        _ => panic!("Impossible semantic error case"),
                    }
                }
                Ok(arr)
            },
            _ => Err("Array of char values expected.".to_string())
        }
    }

    pub fn to_vector_string(&self) -> Result<Vec<String>, String> {

        let datatype = DataType::new(Structure::Scalar, Type::String);

        match self {
            ValueContent::Array(vec) => {
                let mut arr: Vec<String> = Vec::with_capacity(vec.len());
                for val in vec {
                    match val.make_executive_value(&datatype)? {
                        ExecutiveValue::String(s) => arr.push(s),
                        _ => panic!("Impossible semantic error case"),
                    }
                }
                Ok(arr)
            },
            _ => Err("Array of string values expected.".to_string())
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::logic::descriptor::datatype::{DataType, Structure, Type};

    #[test]
    fn test_make_executive_values() {
        
            /* Try for:
            ValueContent::Boolean(true),
            ValueContent::Unsigned(456),
            ValueContent::Integer(-789),
            ValueContent::Real(12.3),
            ValueContent::String("Foo bar".to_string()),
            */

        assert_eq!(ValueContent::Boolean(true).make_executive_value(&DataType::new(Structure::Scalar, Type::Bool)).unwrap(), ExecutiveValue::Bool(true));
        assert_eq!(ValueContent::Boolean(false).make_executive_value(&DataType::new(Structure::Scalar, Type::Bool)).unwrap(), ExecutiveValue::Bool(false));
        assert_eq!(ValueContent::Unsigned(123).make_executive_value(&DataType::new(Structure::Scalar, Type::I8)).unwrap(), ExecutiveValue::I8(123));
    }
}

