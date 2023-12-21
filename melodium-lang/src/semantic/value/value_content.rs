use super::super::common::Reference;
use super::super::declared_parameter::DeclaredParameter;
use super::super::function_call::FunctionCall;
use super::super::requirement::Requirement;
use melodium_common::descriptor::DataType;
use melodium_common::executive::Value as ExecutiveValue;
use std::convert::TryFrom;
use std::sync::{Arc, RwLock};

/// Enum holding value or reference designating the value.
#[derive(Debug)]
pub enum ValueContent {
    Void,
    Boolean(bool),
    Unsigned(u128),
    Integer(i128),
    Real(f64),
    String(String),
    Character(char),
    Byte(u8),
    /// Array, allowing recursive values (in case of vectors).
    Array(Vec<ValueContent>),
    /// Named value, referring to a parameter of the hosting treatment.
    Name(Reference<DeclaredParameter>),
    /// Context reference, referring to a requirement of the hosting treatment, and an inner element.
    ContextReference((Reference<Requirement>, String)),
    /// Function, refering to a function call.
    Function(Arc<RwLock<FunctionCall>>),
}

impl ValueContent {
    pub fn make_executive_value(&self, datatype: &DataType) -> Result<ExecutiveValue, String> {
        match datatype {
            DataType::Undetermined => panic!("Undetermined datatype not possible"),
            DataType::Void => match self {
                ValueContent::Void => Ok(ExecutiveValue::Option(None)),
                _ => Err("Void expected".to_string()),
            },
            DataType::I8 => match self {
                ValueContent::Unsigned(u) => {
                    if let Ok(i) = i8::try_from(*u) {
                        Ok(ExecutiveValue::I8(i))
                    } else {
                        Err("Value too large for i8.".to_string())
                    }
                }
                ValueContent::Integer(i) => {
                    if let Ok(i) = i8::try_from(*i) {
                        Ok(ExecutiveValue::I8(i))
                    } else {
                        Err("Value too large for i8.".to_string())
                    }
                }
                _ => Err("i8 value expected.".to_string()),
            },
            DataType::I16 => match self {
                ValueContent::Unsigned(u) => {
                    if let Ok(i) = i16::try_from(*u) {
                        Ok(ExecutiveValue::I16(i))
                    } else {
                        Err("Value too large for i16.".to_string())
                    }
                }
                ValueContent::Integer(i) => {
                    if let Ok(i) = i16::try_from(*i) {
                        Ok(ExecutiveValue::I16(i))
                    } else {
                        Err("Value too large for i16.".to_string())
                    }
                }
                _ => Err("i16 value expected.".to_string()),
            },
            DataType::I32 => match self {
                ValueContent::Unsigned(u) => {
                    if let Ok(i) = i32::try_from(*u) {
                        Ok(ExecutiveValue::I32(i))
                    } else {
                        Err("Value too large for i32.".to_string())
                    }
                }
                ValueContent::Integer(i) => {
                    if let Ok(i) = i32::try_from(*i) {
                        Ok(ExecutiveValue::I32(i))
                    } else {
                        Err("Value too large for i32.".to_string())
                    }
                }
                _ => Err("i32 value expected.".to_string()),
            },
            DataType::I64 => match self {
                ValueContent::Unsigned(u) => {
                    if let Ok(i) = i64::try_from(*u) {
                        Ok(ExecutiveValue::I64(i))
                    } else {
                        Err("Value too large for i64.".to_string())
                    }
                }
                ValueContent::Integer(i) => {
                    if let Ok(i) = i64::try_from(*i) {
                        Ok(ExecutiveValue::I64(i))
                    } else {
                        Err("Value too large for i64.".to_string())
                    }
                }
                _ => Err("i64 value expected.".to_string()),
            },
            DataType::I128 => match self {
                ValueContent::Unsigned(u) => {
                    if let Ok(i) = i128::try_from(*u) {
                        Ok(ExecutiveValue::I128(i))
                    } else {
                        Err("Value too large for i128.".to_string())
                    }
                }
                ValueContent::Integer(i) => Ok(ExecutiveValue::I128(*i)),
                _ => Err("i128 value expected.".to_string()),
            },
            DataType::U8 => match self {
                ValueContent::Unsigned(u) => {
                    if let Ok(u) = u8::try_from(*u) {
                        Ok(ExecutiveValue::U8(u))
                    } else {
                        Err("Value too large for u8.".to_string())
                    }
                }
                ValueContent::Integer(i) => {
                    if let Ok(u) = u8::try_from(*i) {
                        Ok(ExecutiveValue::U8(u))
                    } else {
                        Err("Value too large for u8.".to_string())
                    }
                }
                _ => Err("u8 value expected.".to_string()),
            },
            DataType::U16 => match self {
                ValueContent::Unsigned(u) => {
                    if let Ok(u) = u16::try_from(*u) {
                        Ok(ExecutiveValue::U16(u))
                    } else {
                        Err("Value too large for u16.".to_string())
                    }
                }
                ValueContent::Integer(i) => {
                    if let Ok(u) = u16::try_from(*i) {
                        Ok(ExecutiveValue::U16(u))
                    } else {
                        Err("Value too large for u16.".to_string())
                    }
                }
                _ => Err("u16 value expected.".to_string()),
            },
            DataType::U32 => match self {
                ValueContent::Unsigned(u) => {
                    if let Ok(u) = u32::try_from(*u) {
                        Ok(ExecutiveValue::U32(u))
                    } else {
                        Err("Value too large for u32.".to_string())
                    }
                }
                ValueContent::Integer(i) => {
                    if let Ok(u) = u32::try_from(*i) {
                        Ok(ExecutiveValue::U32(u))
                    } else {
                        Err("Value too large for u32.".to_string())
                    }
                }
                _ => Err("u32 value expected.".to_string()),
            },
            DataType::U64 => match self {
                ValueContent::Unsigned(u) => {
                    if let Ok(u) = u64::try_from(*u) {
                        Ok(ExecutiveValue::U64(u))
                    } else {
                        Err("Value too large for u64.".to_string())
                    }
                }
                ValueContent::Integer(i) => {
                    if let Ok(u) = u64::try_from(*i) {
                        Ok(ExecutiveValue::U64(u))
                    } else {
                        Err("Value too large for u64.".to_string())
                    }
                }
                _ => Err("u64 value expected.".to_string()),
            },
            DataType::U128 => match self {
                ValueContent::Unsigned(u) => Ok(ExecutiveValue::U128(*u)),
                ValueContent::Integer(i) => {
                    if let Ok(u) = u128::try_from(*i) {
                        Ok(ExecutiveValue::U128(u))
                    } else {
                        Err("Value too large for u128.".to_string())
                    }
                }
                _ => Err("u128 value expected.".to_string()),
            },
            DataType::F32 => match self {
                ValueContent::Unsigned(u) => {
                    // We convert from u16 because f32 cannot hold greater values.
                    if let Ok(u) = u16::try_from(*u) {
                        Ok(ExecutiveValue::F32(f32::from(u)))
                    } else {
                        Err("Value too large for f32.".to_string())
                    }
                }
                ValueContent::Integer(i) => {
                    // We convert from i16 because f32 cannot hold greater values.
                    if let Ok(i) = i16::try_from(*i) {
                        Ok(ExecutiveValue::F32(f32::from(i)))
                    } else {
                        Err("Value too large for f32.".to_string())
                    }
                }
                ValueContent::Real(b) => Ok(ExecutiveValue::F32(*b as f32)),
                _ => Err("f32 value expected.".to_string()),
            },
            DataType::F64 => match self {
                ValueContent::Unsigned(u) => {
                    // We convert from u32 because f64 cannot hold greater values.
                    if let Ok(u) = u32::try_from(*u) {
                        Ok(ExecutiveValue::F64(f64::from(u)))
                    } else {
                        Err("Value too large for f64.".to_string())
                    }
                }
                ValueContent::Integer(i) => {
                    // We convert from i32 because f64 cannot hold greater values.
                    if let Ok(i) = i32::try_from(*i) {
                        Ok(ExecutiveValue::F64(f64::from(i)))
                    } else {
                        Err("Value too large for f64.".to_string())
                    }
                }
                ValueContent::Real(b) => Ok(ExecutiveValue::F64(*b)),
                _ => Err("f64 value expected.".to_string()),
            },
            DataType::Bool => match self {
                ValueContent::Boolean(b) => Ok(ExecutiveValue::Bool(*b)),
                _ => Err("Boolean value expected.".to_string()),
            },
            DataType::Byte => match self {
                ValueContent::Byte(b) => Ok(ExecutiveValue::Byte(*b)),
                ValueContent::Unsigned(u) => {
                    if let Ok(u) = u8::try_from(*u) {
                        Ok(ExecutiveValue::Byte(u))
                    } else {
                        Err("Value too large for byte.".to_string())
                    }
                }
                ValueContent::Integer(i) => {
                    if let Ok(u) = u8::try_from(*i) {
                        Ok(ExecutiveValue::Byte(u))
                    } else {
                        Err("Value too large for byte.".to_string())
                    }
                }
                _ => Err("byte value expected.".to_string()),
            },
            DataType::Char => match self {
                ValueContent::Character(c) => Ok(ExecutiveValue::Char(*c)),
                ValueContent::Unsigned(u) => {
                    if let Ok(u) = u32::try_from(*u) {
                        if let Some(c) = char::from_u32(u) {
                            Ok(ExecutiveValue::Char(c))
                        } else {
                            Err("Value cannot be a char.".to_string())
                        }
                    } else {
                        Err("Value too large for char.".to_string())
                    }
                }
                ValueContent::Integer(i) => {
                    if let Ok(u) = u32::try_from(*i) {
                        if let Some(c) = char::from_u32(u) {
                            Ok(ExecutiveValue::Char(c))
                        } else {
                            Err("Value cannot be a char.".to_string())
                        }
                    } else {
                        Err("Value too large for char.".to_string())
                    }
                }
                _ => Err("char value expected.".to_string()),
            },
            DataType::String => match self {
                ValueContent::String(s) => Ok(ExecutiveValue::String(s.clone())),
                _ => Err("String value expected.".to_string()),
            },
            DataType::Vec(inner_type) => match self {
                ValueContent::Array(vec) => {
                    let mut arr: Vec<ExecutiveValue> = Vec::with_capacity(vec.len());
                    for val in vec {
                        arr.push(val.make_executive_value(&inner_type)?);
                    }
                    Ok(ExecutiveValue::Vec(arr))
                }
                _ => Err("Vector expected".to_string()),
            },
            DataType::Option(inner_type) => match self {
                ValueContent::Void => Ok(ExecutiveValue::Option(None)),
                me => Ok(me.make_executive_value(&inner_type)?),
            },
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use melodium_common::descriptor::DataType;

    #[test]
    fn test_make_executive_values() {
        /* Try for:
        ValueContent::Boolean(true),
        ValueContent::Unsigned(456),
        ValueContent::Integer(-789),
        ValueContent::Real(12.3),
        ValueContent::String("Foo bar".to_string()),
        */

        assert_eq!(
            ValueContent::Boolean(true)
                .make_executive_value(&DataType::Bool)
                .unwrap(),
            ExecutiveValue::Bool(true)
        );
        assert_eq!(
            ValueContent::Boolean(false)
                .make_executive_value(&DataType::Bool)
                .unwrap(),
            ExecutiveValue::Bool(false)
        );
        assert_eq!(
            ValueContent::Unsigned(123)
                .make_executive_value(&DataType::I8)
                .unwrap(),
            ExecutiveValue::I8(123)
        );
    }
}
