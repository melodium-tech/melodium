
use std::sync::{Arc, Weak, RwLock};
use std::convert::TryFrom;
use crate::script::error::ScriptError;
use crate::script::path::Path;
use crate::script::text::{PositionnedString, Position};
use crate::script::text::value::Value as TextValue;
use crate::executive::value::Value as ExecutiveValue;
use crate::logic::descriptor::datatype::{DataType, Structure, Type};
use crate::logic::designer::ValueDesigner;

use super::super::declarative_element::{DeclarativeElement, DeclarativeElementType};
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

    pub fn make_executive_value(&self, datatype: &DataType) -> Result<ExecutiveValue, ScriptError> {

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
                                    Err(ScriptError::semantic("Value too large for u8.".to_string(), self.text.get_position()))
                                }
                            },
                            ValueContent::Integer(i) => {
                                if let Ok(u) = u8::try_from(*i) {
                                    Ok(ExecutiveValue::U8(u))
                                }
                                else {
                                    Err(ScriptError::semantic("Value too large for u8.".to_string(), self.text.get_position()))
                                }
                            },
                            _ => Err(ScriptError::semantic("u8 value expected.".to_string(), self.text.get_position()))
                        },
                    
                    Type::U16 =>
                        match &self {
                            ValueContent::Unsigned(u) => {
                                if let Ok(u) = u16::try_from(*u) {
                                    Ok(ExecutiveValue::U16(u))
                                }
                                else {
                                    Err(ScriptError::semantic("Value too large for u16.".to_string(), self.text.get_position()))
                                }
                            },
                            ValueContent::Integer(i) => {
                                if let Ok(u) = u16::try_from(*i) {
                                    Ok(ExecutiveValue::U16(u))
                                }
                                else {
                                    Err(ScriptError::semantic("Value too large for u16.".to_string(), self.text.get_position()))
                                }
                            },
                            _ => Err(ScriptError::semantic("u16 value expected.".to_string(), self.text.get_position()))
                        },
                    
                    Type::U32 =>
                        match &self {
                            ValueContent::Unsigned(u) => {
                                if let Ok(u) = u32::try_from(*u) {
                                    Ok(ExecutiveValue::U32(u))
                                }
                                else {
                                    Err(ScriptError::semantic("Value too large for u32.".to_string(), self.text.get_position()))
                                }
                            },
                            ValueContent::Integer(i) => {
                                if let Ok(u) = u32::try_from(*i) {
                                    Ok(ExecutiveValue::U32(u))
                                }
                                else {
                                    Err(ScriptError::semantic("Value too large for u32.".to_string(), self.text.get_position()))
                                }
                            },
                            _ => Err(ScriptError::semantic("u32 value expected.".to_string(), self.text.get_position()))
                        },
                    
                    Type::U64 =>
                        match &self {
                            ValueContent::Unsigned(u) => {
                                if let Ok(u) = u64::try_from(*u) {
                                    Ok(ExecutiveValue::U64(u))
                                }
                                else {
                                    Err(ScriptError::semantic("Value too large for u64.".to_string(), self.text.get_position()))
                                }
                            },
                            ValueContent::Integer(i) => {
                                if let Ok(u) = u64::try_from(*i) {
                                    Ok(ExecutiveValue::U64(u))
                                }
                                else {
                                    Err(ScriptError::semantic("Value too large for u64.".to_string(), self.text.get_position()))
                                }
                            },
                            _ => Err(ScriptError::semantic("u64 value expected.".to_string(), self.text.get_position()))
                        },
                    
                    Type::U128 =>
                        match &self {
                            ValueContent::Unsigned(u) => Ok(ExecutiveValue::U128(*u)),
                            ValueContent::Integer(i) => {
                                if let Ok(u) = u128::try_from(*i) {
                                    Ok(ExecutiveValue::U128(u))
                                }
                                else {
                                    Err(ScriptError::semantic("Value too large for u128.".to_string(), self.text.get_position()))
                                }
                            },
                            _ => Err(ScriptError::semantic("u128 value expected.".to_string(), self.text.get_position()))
                        },

                    Type::I8 =>
                        match &self {
                            ValueContent::Unsigned(u) => {
                                if let Ok(i) = i8::try_from(*u) {
                                    Ok(ExecutiveValue::I8(i))
                                }
                                else {
                                    Err(ScriptError::semantic("Value too large for i8.".to_string(), self.text.get_position()))
                                }
                            },
                            ValueContent::Integer(i) => {
                                if let Ok(i) = i8::try_from(*i) {
                                    Ok(ExecutiveValue::I8(i))
                                }
                                else {
                                    Err(ScriptError::semantic("Value too large for i8.".to_string(), self.text.get_position()))
                                }
                            },
                            _ => Err(ScriptError::semantic("i8 value expected.".to_string(), self.text.get_position()))
                        },

                    Type::I16 =>
                        match &self {
                            ValueContent::Unsigned(u) => {
                                if let Ok(i) = i16::try_from(*u) {
                                    Ok(ExecutiveValue::I16(i))
                                }
                                else {
                                    Err(ScriptError::semantic("Value too large for i16.".to_string(), self.text.get_position()))
                                }
                            },
                            ValueContent::Integer(i) => {
                                if let Ok(i) = i16::try_from(*i) {
                                    Ok(ExecutiveValue::I16(i))
                                }
                                else {
                                    Err(ScriptError::semantic("Value too large for i16.".to_string(), self.text.get_position()))
                                }
                            },
                            _ => Err(ScriptError::semantic("i16 value expected.".to_string(), self.text.get_position()))
                        },
                    
                    Type::I32 =>
                        match &self {
                            ValueContent::Unsigned(u) => {
                                if let Ok(i) = i32::try_from(*u) {
                                    Ok(ExecutiveValue::I32(i))
                                }
                                else {
                                    Err(ScriptError::semantic("Value too large for i32.".to_string(), self.text.get_position()))
                                }
                            },
                            ValueContent::Integer(i) => {
                                if let Ok(i) = i32::try_from(*i) {
                                    Ok(ExecutiveValue::I32(i))
                                }
                                else {
                                    Err(ScriptError::semantic("Value too large for i32.".to_string(), self.text.get_position()))
                                }
                            },
                            _ => Err(ScriptError::semantic("i32 value expected.".to_string(), self.text.get_position()))
                        },

                    Type::I64 =>
                        match &self {
                            ValueContent::Unsigned(u) => {
                                if let Ok(i) = i64::try_from(*u) {
                                    Ok(ExecutiveValue::I64(i))
                                }
                                else {
                                    Err(ScriptError::semantic("Value too large for i64.".to_string(), self.text.get_position()))
                                }
                            },
                            ValueContent::Integer(i) => {
                                if let Ok(i) = i64::try_from(*i) {
                                    Ok(ExecutiveValue::I64(i))
                                }
                                else {
                                    Err(ScriptError::semantic("Value too large for i64.".to_string(), self.text.get_position()))
                                }
                            },
                            _ => Err(ScriptError::semantic("i64 value expected.".to_string(), self.text.get_position()))
                        },
                    
                    Type::I128 =>
                        match &self {
                            ValueContent::Unsigned(u) => {
                                if let Ok(i) = i128::try_from(*u) {
                                    Ok(ExecutiveValue::I128(i))
                                }
                                else {
                                    Err(ScriptError::semantic("Value too large for i128.".to_string(), self.text.get_position()))
                                }
                            },
                            ValueContent::Integer(i) => Ok(ExecutiveValue::I128(*i)),
                            _ => Err(ScriptError::semantic("i128 value expected.".to_string(), self.text.get_position()))
                        },
                    
                    Type::F32 =>
                        match &self {
                            ValueContent::Unsigned(u) => {
                                // We convert from u16 because f32 cannot hold greater values.
                                if let Ok(u) = u16::try_from(*u) {
                                    Ok(ExecutiveValue::F32(f32::from(u)))
                                }
                                else {
                                    Err(ScriptError::semantic("Value too large for f32.".to_string(), self.text.get_position()))
                                }
                            },
                            ValueContent::Integer(i) => {
                                // We convert from i16 because f32 cannot hold greater values.
                                if let Ok(i) = i16::try_from(*i) {
                                    Ok(ExecutiveValue::F32(f32::from(i)))
                                }
                                else {
                                    Err(ScriptError::semantic("Value too large for f32.".to_string(), self.text.get_position()))
                                }
                            },
                            ValueContent::Real(b) => Ok(ExecutiveValue::F32(*b as f32)),
                            _ => Err(ScriptError::semantic("f32 value expected.".to_string(), self.text.get_position()))
                        },
                    
                    Type::F64 =>
                        match &self {
                            ValueContent::Unsigned(u) => {
                                // We convert from u32 because f64 cannot hold greater values.
                                if let Ok(u) = u32::try_from(*u) {
                                    Ok(ExecutiveValue::F64(f64::from(u)))
                                }
                                else {
                                    Err(ScriptError::semantic("Value too large for f64.".to_string(), self.text.get_position()))
                                }
                            },
                            ValueContent::Integer(i) => {
                                // We convert from i32 because f64 cannot hold greater values.
                                if let Ok(i) = i32::try_from(*i) {
                                    Ok(ExecutiveValue::F64(f64::from(i)))
                                }
                                else {
                                    Err(ScriptError::semantic("Value too large for f64.".to_string(), self.text.get_position()))
                                }
                            },
                            ValueContent::Real(b) => Ok(ExecutiveValue::F64(*b)),
                            _ => Err(ScriptError::semantic("f64 value expected.".to_string(), self.text.get_position()))
                        },
                    
                    Type::Bool =>
                        match &self {
                            ValueContent::Boolean(b) => Ok(ExecutiveValue::Bool(*b)),
                            _ => Err(ScriptError::semantic("Boolean value expected.".to_string(), self.text.get_position()))
                        },
                    
                    Type::Byte =>
                        match &self {
                            ValueContent::Unsigned(u) => {
                                if let Ok(u) = u8::try_from(*u) {
                                    Ok(ExecutiveValue::Byte(u))
                                }
                                else {
                                    Err(ScriptError::semantic("Value too large for byte.".to_string(), self.text.get_position()))
                                }
                            },
                            ValueContent::Integer(i) => {
                                if let Ok(u) = u8::try_from(*i) {
                                    Ok(ExecutiveValue::Byte(u))
                                }
                                else {
                                    Err(ScriptError::semantic("Value too large for byte.".to_string(), self.text.get_position()))
                                }
                            },
                            _ => Err(ScriptError::semantic("byte value expected.".to_string(), self.text.get_position()))
                        },
                    
                    Type::Char =>
                        match &self {
                            ValueContent::Unsigned(u) => {
                                if let Ok(u) = u32::try_from(*u) {
                                    if let Some(c) = char::from_u32(u) {
                                        Ok(ExecutiveValue::Char(c))
                                    }
                                    else {
                                        Err(ScriptError::semantic("Value cannot be a char.".to_string(), self.text.get_position()))
                                    }
                                }
                                else {
                                    Err(ScriptError::semantic("Value too large for char.".to_string(), self.text.get_position()))
                                }
                            },
                            ValueContent::Integer(i) => {
                                if let Ok(u) = u32::try_from(*i) {
                                    if let Some(c) = char::from_u32(u) {
                                        Ok(ExecutiveValue::Char(c))
                                    }
                                    else {
                                        Err(ScriptError::semantic("Value cannot be a char.".to_string(), self.text.get_position()))
                                    }
                                }
                                else {
                                    Err(ScriptError::semantic("Value too large for char.".to_string(), self.text.get_position()))
                                }
                            },
                            _ => Err(ScriptError::semantic("char value expected.".to_string(), self.text.get_position()))
                        },

                    Type::String => 
                        match &self {
                            ValueContent::String(s) => Ok(ExecutiveValue::String(s.clone())),
                            _ => Err(ScriptError::semantic("String value expected.".to_string(), self.text.get_position()))
                        },

                }
            },
            Structure::Vector => {
                match datatype.r#type() {
                    Type::Boolean => 
                        if let Some(vec) = self.content.to_vector_bool() {
                            Ok(ExecutiveValue::VecBoolean(vec))
                        }
                        else {
                            Err(ScriptError::semantic("Array of boolean values expected.".to_string(), self.text.get_position()))
                        },
                    Type::Integer =>
                        if let Some(vec) = self.content.to_vector_integer() {
                            Ok(ExecutiveValue::VecInteger(vec))
                        }
                        else {
                            Err(ScriptError::semantic("Array of integer values expected.".to_string(), self.text.get_position()))
                        },
                    Type::Real => 
                        if let Some(vec) = self.content.to_vector_real() {
                            Ok(ExecutiveValue::VecReal(vec))
                        }
                        else {
                            Err(ScriptError::semantic("Array of real values expected.".to_string(), self.text.get_position()))
                        },
                    Type::String => 
                        if let Some(vec) = self.content.to_vector_string() {
                            Ok(ExecutiveValue::VecString(vec))
                        }
                        else {
                            Err(ScriptError::semantic("Array of string values expected.".to_string(), self.text.get_position()))
                        },
                }
            },
        }
    }

    pub fn to_vector_bool(&self) -> Option<Vec<bool>> {
        match self {
            ValueContent::Array(vec) => {
                let mut arr: Vec<bool> = Vec::with_capacity(vec.len());
                for val in vec {
                    match val {
                        ValueContent::Boolean(b) => arr.push(*b),
                        _ => return None,
                    }
                }
                Some(arr)
            },
            _ => None
        }
    }

    pub fn to_vector_integer(&self) -> Option<Vec<i64>> {
        match self {
            ValueContent::Array(vec) => {
                let mut arr: Vec<i64> = Vec::with_capacity(vec.len());
                for val in vec {
                    match val {
                        ValueContent::Integer(i) => arr.push(*i),
                        _ => return None,
                    }
                }
                Some(arr)
            },
            _ => None
        }
    }

    pub fn to_vector_real(&self) -> Option<Vec<f64>> {
        match self {
            ValueContent::Array(vec) => {
                let mut arr: Vec<f64> = Vec::with_capacity(vec.len());
                for val in vec {
                    match val {
                        ValueContent::Real(r) => arr.push(*r),
                        _ => return None,
                    }
                }
                Some(arr)
            },
            _ => None
        }
    }

    pub fn to_vector_string(&self) -> Option<Vec<String>> {
        match self {
            ValueContent::Array(vec) => {
                let mut arr: Vec<String> = Vec::with_capacity(vec.len());
                for val in vec {
                    match val {
                        ValueContent::String(s) => arr.push(s.clone()),
                        _ => return None,
                    }
                }
                Some(arr)
            },
            _ => None
        }
    }
}

