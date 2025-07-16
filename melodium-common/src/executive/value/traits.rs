use super::Value;
use crate::executive::DataTrait;
use core::{convert::TryInto, fmt::Debug, hash::Hash};
use erased_serde::Serialize;
use serde::ser::SerializeSeq;

impl DataTrait for Value {
    fn to_i8(&self) -> i8 {
        match self {
            Value::I8(val) => *val,
            Value::Data(obj) => obj.to_i8(),
            other => panic!("ToI8 not supported for {}", other.datatype()),
        }
    }

    fn to_i16(&self) -> i16 {
        match self {
            Value::I8(val) => *val as i16,
            Value::I16(val) => *val,
            Value::U8(val) => *val as i16,
            Value::Data(obj) => obj.to_i16(),
            other => panic!("ToI16 not supported for {}", other.datatype()),
        }
    }

    fn to_i32(&self) -> i32 {
        match self {
            Value::I8(val) => *val as i32,
            Value::I16(val) => *val as i32,
            Value::I32(val) => *val,
            Value::U8(val) => *val as i32,
            Value::U16(val) => *val as i32,
            Value::Data(obj) => obj.to_i32(),
            other => panic!("ToI32 not supported for {}", other.datatype()),
        }
    }

    fn to_i64(&self) -> i64 {
        match self {
            Value::I8(val) => *val as i64,
            Value::I16(val) => *val as i64,
            Value::I32(val) => *val as i64,
            Value::I64(val) => *val,
            Value::U8(val) => *val as i64,
            Value::U16(val) => *val as i64,
            Value::U32(val) => *val as i64,
            Value::Data(obj) => obj.to_i64(),
            other => panic!("ToI64 not supported for {}", other.datatype()),
        }
    }

    fn to_i128(&self) -> i128 {
        match self {
            Value::I8(val) => *val as i128,
            Value::I16(val) => *val as i128,
            Value::I32(val) => *val as i128,
            Value::I64(val) => *val as i128,
            Value::U8(val) => *val as i128,
            Value::U16(val) => *val as i128,
            Value::U32(val) => *val as i128,
            Value::U64(val) => *val as i128,
            Value::I128(val) => *val,
            Value::Data(obj) => obj.to_i128(),
            other => panic!("ToI128 not supported for {}", other.datatype()),
        }
    }

    fn to_u8(&self) -> u8 {
        match self {
            Value::U8(val) => *val,
            Value::Byte(val) => *val,
            Value::Data(obj) => obj.to_u8(),
            other => panic!("ToU8 not supported for {}", other.datatype()),
        }
    }

    fn to_u16(&self) -> u16 {
        match self {
            Value::U8(val) => *val as u16,
            Value::U16(val) => *val,
            Value::Data(obj) => obj.to_u16(),
            other => panic!("ToU16 not supported for {}", other.datatype()),
        }
    }

    fn to_u32(&self) -> u32 {
        match self {
            Value::U8(val) => *val as u32,
            Value::U16(val) => *val as u32,
            Value::U32(val) => *val,
            Value::Data(obj) => obj.to_u32(),
            other => panic!("ToU32 not supported for {}", other.datatype()),
        }
    }

    fn to_u64(&self) -> u64 {
        match self {
            Value::U8(val) => *val as u64,
            Value::U16(val) => *val as u64,
            Value::U32(val) => *val as u64,
            Value::U64(val) => *val,
            Value::Data(obj) => obj.to_u64(),
            other => panic!("ToU64 not supported for {}", other.datatype()),
        }
    }

    fn to_u128(&self) -> u128 {
        match self {
            Value::U8(val) => *val as u128,
            Value::U16(val) => *val as u128,
            Value::U32(val) => *val as u128,
            Value::U64(val) => *val as u128,
            Value::U128(val) => *val,
            Value::Data(obj) => obj.to_u128(),
            other => panic!("ToU128 not supported for {}", other.datatype()),
        }
    }

    fn to_f32(&self) -> f32 {
        match self {
            Value::I8(val) => *val as f32,
            Value::I16(val) => *val as f32,
            Value::I32(val) => *val as f32,
            Value::I64(val) => *val as f32,
            Value::I128(val) => *val as f32,
            Value::U8(val) => *val as f32,
            Value::U16(val) => *val as f32,
            Value::U32(val) => *val as f32,
            Value::U64(val) => *val as f32,
            Value::U128(val) => *val as f32,
            Value::F32(val) => *val,
            Value::F64(val) => *val as f32,
            Value::Data(obj) => obj.to_f32(),
            other => panic!("ToF32 not supported for {}", other.datatype()),
        }
    }

    fn to_f64(&self) -> f64 {
        match self {
            Value::I8(val) => *val as f64,
            Value::I16(val) => *val as f64,
            Value::I32(val) => *val as f64,
            Value::I64(val) => *val as f64,
            Value::I128(val) => *val as f64,
            Value::U8(val) => *val as f64,
            Value::U16(val) => *val as f64,
            Value::U32(val) => *val as f64,
            Value::U64(val) => *val as f64,
            Value::U128(val) => *val as f64,
            Value::F32(val) => *val as f64,
            Value::F64(val) => *val,
            Value::Data(obj) => obj.to_f64(),
            other => panic!("ToF64 not supported for {}", other.datatype()),
        }
    }

    fn to_bool(&self) -> bool {
        match self {
            Value::I8(val) => *val != 0,
            Value::I16(val) => *val != 0,
            Value::I32(val) => *val != 0,
            Value::I64(val) => *val != 0,
            Value::I128(val) => *val != 0,
            Value::U8(val) => *val != 0,
            Value::U16(val) => *val != 0,
            Value::U32(val) => *val != 0,
            Value::U64(val) => *val != 0,
            Value::U128(val) => *val != 0,
            Value::Bool(val) => *val,
            Value::Byte(val) => *val != 0,
            Value::Data(obj) => obj.to_bool(),
            other => panic!("ToBool not supported for {}", other.datatype()),
        }
    }

    fn to_byte(&self) -> u8 {
        match self {
            Value::U8(val) => *val,
            Value::Bool(val) => *val as u8,
            Value::Byte(val) => *val,
            Value::Data(obj) => obj.to_byte(),
            other => panic!("ToByte not supported for {}", other.datatype()),
        }
    }

    fn to_char(&self) -> char {
        match self {
            Value::Char(val) => *val,
            Value::Data(obj) => obj.to_char(),
            other => panic!("ToChar not supported for {}", other.datatype()),
        }
    }

    fn to_string(&self) -> String {
        match self {
            Value::Void(_val) => String::new(),
            Value::I8(val) => val.to_string(),
            Value::I16(val) => val.to_string(),
            Value::I32(val) => val.to_string(),
            Value::I64(val) => val.to_string(),
            Value::I128(val) => val.to_string(),
            Value::U8(val) => val.to_string(),
            Value::U16(val) => val.to_string(),
            Value::U32(val) => val.to_string(),
            Value::U64(val) => val.to_string(),
            Value::U128(val) => val.to_string(),
            Value::F32(val) => val.to_string(),
            Value::F64(val) => val.to_string(),
            Value::Bool(val) => val.to_string(),
            Value::Byte(val) => hex::encode([*val]),
            Value::Char(val) => val.to_string(),
            Value::String(val) => val.clone(),
            Value::Data(obj) => obj.to_string(),
            other => panic!("ToString not supported for {}", other.datatype()),
        }
    }

    fn try_to_i8(&self) -> Option<i8> {
        match self {
            Value::I8(val) => Some(*val),
            Value::I16(val) => TryInto::<i8>::try_into(*val).ok(),

            Value::I32(val) => TryInto::<i8>::try_into(*val).ok(),
            Value::I64(val) => TryInto::<i8>::try_into(*val).ok(),
            Value::I128(val) => TryInto::<i8>::try_into(*val).ok(),
            Value::U8(val) => TryInto::<i8>::try_into(*val).ok(),
            Value::U16(val) => TryInto::<i8>::try_into(*val).ok(),
            Value::U32(val) => TryInto::<i8>::try_into(*val).ok(),
            Value::U64(val) => TryInto::<i8>::try_into(*val).ok(),
            Value::U128(val) => TryInto::<i8>::try_into(*val).ok(),
            Value::F32(val) => {
                if val.is_finite() && *val <= i8::MAX as f32 && *val >= i8::MIN as f32 {
                    Some(*val as i8)
                } else {
                    None
                }
            }

            Value::F64(val) => {
                if val.is_finite() && *val <= i8::MAX as f64 && *val >= i8::MIN as f64 {
                    Some(*val as i8)
                } else {
                    None
                }
            }

            Value::Data(obj) => obj.try_to_i8(),

            other => panic!("TryToI8 not supported for {}", other.datatype()),
        }
    }

    fn try_to_i16(&self) -> Option<i16> {
        match self {
            Value::I8(val) => Some(*val as i16),
            Value::I16(val) => Some(*val),
            Value::I32(val) => TryInto::<i16>::try_into(*val).ok(),
            Value::I64(val) => TryInto::<i16>::try_into(*val).ok(),
            Value::I128(val) => TryInto::<i16>::try_into(*val).ok(),
            Value::U8(val) => TryInto::<i16>::try_into(*val).ok(),
            Value::U16(val) => TryInto::<i16>::try_into(*val).ok(),
            Value::U32(val) => TryInto::<i16>::try_into(*val).ok(),
            Value::U64(val) => TryInto::<i16>::try_into(*val).ok(),
            Value::U128(val) => TryInto::<i16>::try_into(*val).ok(),
            Value::F32(val) => {
                if val.is_finite() && *val <= i16::MAX as f32 && *val >= i16::MIN as f32 {
                    Some(*val as i16)
                } else {
                    None
                }
            }

            Value::F64(val) => {
                if val.is_finite() && *val <= i16::MAX as f64 && *val >= i16::MIN as f64 {
                    Some(*val as i16)
                } else {
                    None
                }
            }

            Value::Data(obj) => obj.try_to_i16(),

            other => panic!("TryToI16 not supported for {}", other.datatype()),
        }
    }

    fn try_to_i32(&self) -> Option<i32> {
        match self {
            Value::I8(val) => Some(*val as i32),
            Value::I16(val) => Some(*val as i32),
            Value::I32(val) => Some(*val),
            Value::I64(val) => TryInto::<i32>::try_into(*val).ok(),
            Value::I128(val) => TryInto::<i32>::try_into(*val).ok(),
            Value::U8(val) => TryInto::<i32>::try_into(*val).ok(),
            Value::U16(val) => TryInto::<i32>::try_into(*val).ok(),
            Value::U32(val) => TryInto::<i32>::try_into(*val).ok(),
            Value::U64(val) => TryInto::<i32>::try_into(*val).ok(),
            Value::U128(val) => TryInto::<i32>::try_into(*val).ok(),
            Value::F32(val) => {
                if val.is_finite() && *val <= i32::MAX as f32 && *val >= i32::MIN as f32 {
                    Some(*val as i32)
                } else {
                    None
                }
            }

            Value::F64(val) => {
                if val.is_finite() && *val <= i32::MAX as f64 && *val >= i32::MIN as f64 {
                    Some(*val as i32)
                } else {
                    None
                }
            }

            Value::Data(obj) => obj.try_to_i32(),

            other => panic!("TryToI32 not supported for {}", other.datatype()),
        }
    }

    fn try_to_i64(&self) -> Option<i64> {
        match self {
            Value::I8(val) => Some(*val as i64),
            Value::I16(val) => Some(*val as i64),
            Value::I32(val) => Some(*val as i64),
            Value::I64(val) => Some(*val),
            Value::I128(val) => TryInto::<i64>::try_into(*val).ok(),
            Value::U8(val) => TryInto::<i64>::try_into(*val).ok(),
            Value::U16(val) => TryInto::<i64>::try_into(*val).ok(),
            Value::U32(val) => TryInto::<i64>::try_into(*val).ok(),
            Value::U64(val) => TryInto::<i64>::try_into(*val).ok(),
            Value::U128(val) => TryInto::<i64>::try_into(*val).ok(),
            Value::F32(val) => {
                if val.is_finite() && *val <= i64::MAX as f32 && *val >= i64::MIN as f32 {
                    Some(*val as i64)
                } else {
                    None
                }
            }

            Value::F64(val) => {
                if val.is_finite() && *val <= i64::MAX as f64 && *val >= i64::MIN as f64 {
                    Some(*val as i64)
                } else {
                    None
                }
            }

            Value::Data(obj) => obj.try_to_i64(),

            other => panic!("TryToI64 not supported for {}", other.datatype()),
        }
    }

    fn try_to_i128(&self) -> Option<i128> {
        match self {
            Value::I8(val) => Some(*val as i128),
            Value::I16(val) => Some(*val as i128),
            Value::I32(val) => Some(*val as i128),
            Value::I64(val) => Some(*val as i128),
            Value::I128(val) => Some(*val),
            Value::U8(val) => TryInto::<i128>::try_into(*val).ok(),
            Value::U16(val) => TryInto::<i128>::try_into(*val).ok(),
            Value::U32(val) => TryInto::<i128>::try_into(*val).ok(),
            Value::U64(val) => TryInto::<i128>::try_into(*val).ok(),
            Value::U128(val) => TryInto::<i128>::try_into(*val).ok(),
            Value::F32(val) => {
                if val.is_finite() && *val <= i128::MAX as f32 && *val >= i128::MIN as f32 {
                    Some(*val as i128)
                } else {
                    None
                }
            }

            Value::F64(val) => {
                if val.is_finite() && *val <= i128::MAX as f64 && *val >= i128::MIN as f64 {
                    Some(*val as i128)
                } else {
                    None
                }
            }

            Value::Data(obj) => obj.try_to_i128(),

            other => panic!("TryToI128 not supported for {}", other.datatype()),
        }
    }

    fn try_to_u8(&self) -> Option<u8> {
        match self {
            Value::I8(val) => TryInto::<u8>::try_into(*val).ok(),
            Value::I16(val) => TryInto::<u8>::try_into(*val).ok(),
            Value::I32(val) => TryInto::<u8>::try_into(*val).ok(),
            Value::I64(val) => TryInto::<u8>::try_into(*val).ok(),
            Value::I128(val) => TryInto::<u8>::try_into(*val).ok(),
            Value::U8(val) => Some(*val),
            Value::U16(val) => TryInto::<u8>::try_into(*val).ok(),
            Value::U32(val) => TryInto::<u8>::try_into(*val).ok(),
            Value::U64(val) => TryInto::<u8>::try_into(*val).ok(),
            Value::U128(val) => TryInto::<u8>::try_into(*val).ok(),
            Value::F32(val) => {
                if val.is_finite() && *val <= u8::MAX as f32 && *val >= u8::MIN as f32 {
                    Some(*val as u8)
                } else {
                    None
                }
            }

            Value::F64(val) => {
                if val.is_finite() && *val <= u8::MAX as f64 && *val >= u8::MIN as f64 {
                    Some(*val as u8)
                } else {
                    None
                }
            }

            Value::Data(obj) => obj.try_to_u8(),

            other => panic!("TryToU8 not supported for {}", other.datatype()),
        }
    }

    fn try_to_u16(&self) -> Option<u16> {
        match self {
            Value::I8(val) => TryInto::<u16>::try_into(*val).ok(),
            Value::I16(val) => TryInto::<u16>::try_into(*val).ok(),
            Value::I32(val) => TryInto::<u16>::try_into(*val).ok(),
            Value::I64(val) => TryInto::<u16>::try_into(*val).ok(),
            Value::I128(val) => TryInto::<u16>::try_into(*val).ok(),
            Value::U8(val) => Some(*val as u16),
            Value::U16(val) => Some(*val),
            Value::U32(val) => TryInto::<u16>::try_into(*val).ok(),
            Value::U64(val) => TryInto::<u16>::try_into(*val).ok(),
            Value::U128(val) => TryInto::<u16>::try_into(*val).ok(),
            Value::F32(val) => {
                if val.is_finite() && *val <= u16::MAX as f32 && *val >= u16::MIN as f32 {
                    Some(*val as u16)
                } else {
                    None
                }
            }

            Value::F64(val) => {
                if val.is_finite() && *val <= u16::MAX as f64 && *val >= u16::MIN as f64 {
                    Some(*val as u16)
                } else {
                    None
                }
            }

            Value::Data(obj) => obj.try_to_u16(),

            other => panic!("TryToU16 not supported for {}", other.datatype()),
        }
    }

    fn try_to_u32(&self) -> Option<u32> {
        match self {
            Value::I8(val) => TryInto::<u32>::try_into(*val).ok(),
            Value::I16(val) => TryInto::<u32>::try_into(*val).ok(),
            Value::I32(val) => TryInto::<u32>::try_into(*val).ok(),
            Value::I64(val) => TryInto::<u32>::try_into(*val).ok(),
            Value::I128(val) => TryInto::<u32>::try_into(*val).ok(),
            Value::U8(val) => Some(*val as u32),
            Value::U16(val) => Some(*val as u32),
            Value::U32(val) => Some(*val),
            Value::U64(val) => TryInto::<u32>::try_into(*val).ok(),
            Value::U128(val) => TryInto::<u32>::try_into(*val).ok(),
            Value::F32(val) => {
                if val.is_finite() && *val <= u32::MAX as f32 && *val >= u32::MIN as f32 {
                    Some(*val as u32)
                } else {
                    None
                }
            }

            Value::F64(val) => {
                if val.is_finite() && *val <= u32::MAX as f64 && *val >= u32::MIN as f64 {
                    Some(*val as u32)
                } else {
                    None
                }
            }

            Value::Data(obj) => obj.try_to_u32(),

            other => panic!("TryToU32 not supported for {}", other.datatype()),
        }
    }

    fn try_to_u64(&self) -> Option<u64> {
        match self {
            Value::I8(val) => TryInto::<u64>::try_into(*val).ok(),
            Value::I16(val) => TryInto::<u64>::try_into(*val).ok(),
            Value::I32(val) => TryInto::<u64>::try_into(*val).ok(),
            Value::I64(val) => TryInto::<u64>::try_into(*val).ok(),
            Value::I128(val) => TryInto::<u64>::try_into(*val).ok(),
            Value::U8(val) => Some(*val as u64),
            Value::U16(val) => Some(*val as u64),
            Value::U32(val) => Some(*val as u64),
            Value::U64(val) => Some(*val),
            Value::U128(val) => TryInto::<u64>::try_into(*val).ok(),
            Value::F32(val) => {
                if val.is_finite() && *val <= u64::MAX as f32 && *val >= u64::MIN as f32 {
                    Some(*val as u64)
                } else {
                    None
                }
            }

            Value::F64(val) => {
                if val.is_finite() && *val <= u64::MAX as f64 && *val >= u64::MIN as f64 {
                    Some(*val as u64)
                } else {
                    None
                }
            }

            Value::Data(obj) => obj.try_to_u64(),

            other => panic!("TryToU64 not supported for {}", other.datatype()),
        }
    }

    fn try_to_u128(&self) -> Option<u128> {
        match self {
            Value::I8(val) => TryInto::<u128>::try_into(*val).ok(),
            Value::I16(val) => TryInto::<u128>::try_into(*val).ok(),
            Value::I32(val) => TryInto::<u128>::try_into(*val).ok(),
            Value::I64(val) => TryInto::<u128>::try_into(*val).ok(),
            Value::I128(val) => TryInto::<u128>::try_into(*val).ok(),
            Value::U8(val) => Some(*val as u128),
            Value::U16(val) => Some(*val as u128),
            Value::U32(val) => Some(*val as u128),
            Value::U64(val) => Some(*val as u128),
            Value::U128(val) => Some(*val),
            Value::F32(val) => {
                if val.is_finite() && *val <= u128::MAX as f32 && *val >= u128::MIN as f32 {
                    Some(*val as u128)
                } else {
                    None
                }
            }

            Value::F64(val) => {
                if val.is_finite() && *val <= u128::MAX as f64 && *val >= u128::MIN as f64 {
                    Some(*val as u128)
                } else {
                    None
                }
            }

            Value::Data(obj) => obj.try_to_u128(),

            other => panic!("TryToU128 not supported for {}", other.datatype()),
        }
    }

    fn try_to_f32(&self) -> Option<f32> {
        match self {
            Value::I8(val) => Some(*val as f32),
            Value::I16(val) => Some(*val as f32),
            Value::I32(val) => Some(*val as f32),
            Value::I64(val) => Some(*val as f32),
            Value::I128(val) => Some(*val as f32),
            Value::U8(val) => Some(*val as f32),
            Value::U16(val) => Some(*val as f32),
            Value::U32(val) => Some(*val as f32),
            Value::U64(val) => Some(*val as f32),
            Value::U128(val) => Some(*val as f32),
            Value::F32(val) => Some(*val),
            Value::F64(val) => Some(*val as f32),
            Value::Data(obj) => obj.try_to_f32(),
            other => panic!("TryToF32 not supported for {}", other.datatype()),
        }
    }

    fn try_to_f64(&self) -> Option<f64> {
        match self {
            Value::I8(val) => Some(*val as f64),
            Value::I16(val) => Some(*val as f64),
            Value::I32(val) => Some(*val as f64),
            Value::I64(val) => Some(*val as f64),
            Value::I128(val) => Some(*val as f64),
            Value::U8(val) => Some(*val as f64),
            Value::U16(val) => Some(*val as f64),
            Value::U32(val) => Some(*val as f64),
            Value::U64(val) => Some(*val as f64),
            Value::U128(val) => Some(*val as f64),
            Value::F32(val) => Some(*val as f64),
            Value::F64(val) => Some(*val),
            Value::Data(obj) => obj.try_to_f64(),
            other => panic!("TryToF32 not supported for {}", other.datatype()),
        }
    }

    fn try_to_bool(&self) -> Option<bool> {
        match self {
            Value::I8(val) => Some(*val != 0),
            Value::I16(val) => Some(*val != 0),
            Value::I32(val) => Some(*val != 0),
            Value::I64(val) => Some(*val != 0),
            Value::I128(val) => Some(*val != 0),
            Value::U8(val) => Some(*val != 0),
            Value::U16(val) => Some(*val != 0),
            Value::U32(val) => Some(*val != 0),
            Value::U64(val) => Some(*val != 0),
            Value::U128(val) => Some(*val != 0),
            Value::Bool(val) => Some(*val),
            Value::Byte(val) => Some(*val != 0),
            Value::Data(obj) => obj.try_to_bool(),
            other => panic!("TryToBool not supported for {}", other.datatype()),
        }
    }

    fn try_to_byte(&self) -> Option<u8> {
        match self {
            Value::U8(val) => Some(*val),
            Value::Bool(val) => Some(*val as u8),
            Value::Byte(val) => Some(*val),
            Value::Data(obj) => obj.try_to_byte(),
            other => panic!("TryToByte not supported for {}", other.datatype()),
        }
    }

    fn try_to_char(&self) -> Option<char> {
        match self {
            Value::Char(val) => Some(*val),
            Value::Data(obj) => obj.try_to_char(),
            other => panic!("TryToChar not supported for {}", other.datatype()),
        }
    }

    fn try_to_string(&self) -> Option<String> {
        match self {
            Value::Void(_val) => Some(String::new()),
            Value::I8(val) => Some(val.to_string()),
            Value::I16(val) => Some(val.to_string()),
            Value::I32(val) => Some(val.to_string()),
            Value::I64(val) => Some(val.to_string()),
            Value::I128(val) => Some(val.to_string()),
            Value::U8(val) => Some(val.to_string()),
            Value::U16(val) => Some(val.to_string()),
            Value::U32(val) => Some(val.to_string()),
            Value::U64(val) => Some(val.to_string()),
            Value::U128(val) => Some(val.to_string()),
            Value::F32(val) => Some(val.to_string()),
            Value::F64(val) => Some(val.to_string()),
            Value::Bool(val) => Some(val.to_string()),
            Value::Byte(val) => Some(hex::encode([*val])),
            Value::Char(val) => Some(val.to_string()),
            Value::String(val) => Some(val.clone()),
            Value::Data(obj) => obj.try_to_string(),
            other => panic!("TryToString not supported for {}", other.datatype()),
        }
    }

    fn saturating_to_i8(&self) -> i8 {
        match self {
            Value::I8(val) => *val,
            Value::I16(val) => match *val {
                val if val < i8::MIN as i16 => i8::MIN,
                val if val > i8::MAX as i16 => i8::MAX,
                val => val as i8,
            },
            Value::I32(val) => match *val {
                val if val < i8::MIN as i32 => i8::MIN,
                val if val > i8::MAX as i32 => i8::MAX,
                val => val as i8,
            },
            Value::I64(val) => match *val {
                val if val < i8::MIN as i64 => i8::MIN,
                val if val > i8::MAX as i64 => i8::MAX,
                val => val as i8,
            },
            Value::I128(val) => match *val {
                val if val < i8::MIN as i128 => i8::MIN,
                val if val > i8::MAX as i128 => i8::MAX,
                val => val as i8,
            },
            Value::U8(val) => match *val {
                val if val > i8::MAX as u8 => i8::MAX,
                val => val as i8,
            },
            Value::U16(val) => match *val {
                val if val > i8::MAX as u16 => i8::MAX,
                val => val as i8,
            },
            Value::U32(val) => match *val {
                val if val > i8::MAX as u32 => i8::MAX,
                val => val as i8,
            },
            Value::U64(val) => match *val {
                val if val > i8::MAX as u64 => i8::MAX,
                val => val as i8,
            },
            Value::U128(val) => match *val {
                val if val > i8::MAX as u128 => i8::MAX,
                val => val as i8,
            },
            Value::F32(val) => {
                if val.is_nan() {
                    0
                } else if *val < i8::MIN as f32 {
                    i8::MIN
                } else if *val > i8::MAX as f32 {
                    i8::MAX
                } else {
                    *val as i8
                }
            }
            Value::F64(val) => {
                if val.is_nan() {
                    0
                } else if *val < i8::MIN as f64 {
                    i8::MIN
                } else if *val > i8::MAX as f64 {
                    i8::MAX
                } else {
                    *val as i8
                }
            }
            Value::Data(obj) => obj.saturating_to_i8(),
            other => panic!("SaturatingToI8 not supported for {}", other.datatype()),
        }
    }

    fn saturating_to_i16(&self) -> i16 {
        match self {
            Value::I8(val) => *val as i16,
            Value::I16(val) => *val,
            Value::I32(val) => match *val {
                val if val < i16::MIN as i32 => i16::MIN,
                val if val > i16::MAX as i32 => i16::MAX,
                val => val as i16,
            },
            Value::I64(val) => match *val {
                val if val < i16::MIN as i64 => i16::MIN,
                val if val > i16::MAX as i64 => i16::MAX,
                val => val as i16,
            },
            Value::I128(val) => match *val {
                val if val < i16::MIN as i128 => i16::MIN,
                val if val > i16::MAX as i128 => i16::MAX,
                val => val as i16,
            },
            Value::U8(val) => *val as i16,
            Value::U16(val) => match *val {
                val if val > i16::MAX as u16 => i16::MAX,
                val => val as i16,
            },
            Value::U32(val) => match *val {
                val if val > i16::MAX as u32 => i16::MAX,
                val => val as i16,
            },
            Value::U64(val) => match *val {
                val if val > i16::MAX as u64 => i16::MAX,
                val => val as i16,
            },
            Value::U128(val) => match *val {
                val if val > i16::MAX as u128 => i16::MAX,
                val => val as i16,
            },
            Value::F32(val) => {
                if val.is_nan() {
                    0
                } else if *val < i16::MIN as f32 {
                    i16::MIN
                } else if *val > i16::MAX as f32 {
                    i16::MAX
                } else {
                    *val as i16
                }
            }
            Value::F64(val) => {
                if val.is_nan() {
                    0
                } else if *val < i16::MIN as f64 {
                    i16::MIN
                } else if *val > i16::MAX as f64 {
                    i16::MAX
                } else {
                    *val as i16
                }
            }
            Value::Data(obj) => obj.saturating_to_i16(),
            other => panic!("SaturatingToI16 not supported for {}", other.datatype()),
        }
    }

    fn saturating_to_i32(&self) -> i32 {
        match self {
            Value::I8(val) => *val as i32,
            Value::I16(val) => *val as i32,
            Value::I32(val) => *val,
            Value::I64(val) => match *val {
                val if val < i32::MIN as i64 => i32::MIN,
                val if val > i32::MAX as i64 => i32::MAX,
                val => val as i32,
            },
            Value::I128(val) => match *val {
                val if val < i32::MIN as i128 => i32::MIN,
                val if val > i32::MAX as i128 => i32::MAX,
                val => val as i32,
            },
            Value::U8(val) => *val as i32,
            Value::U16(val) => *val as i32,
            Value::U32(val) => match *val {
                val if val > i32::MAX as u32 => i32::MAX,
                val => val as i32,
            },
            Value::U64(val) => match *val {
                val if val > i32::MAX as u64 => i32::MAX,
                val => val as i32,
            },
            Value::U128(val) => match *val {
                val if val > i32::MAX as u128 => i32::MAX,
                val => val as i32,
            },
            Value::F32(val) => {
                if val.is_nan() {
                    0
                } else if *val < i32::MIN as f32 {
                    i32::MIN
                } else if *val > i32::MAX as f32 {
                    i32::MAX
                } else {
                    *val as i32
                }
            }
            Value::F64(val) => {
                if val.is_nan() {
                    0
                } else if *val < i32::MIN as f64 {
                    i32::MIN
                } else if *val > i32::MAX as f64 {
                    i32::MAX
                } else {
                    *val as i32
                }
            }
            Value::Data(obj) => obj.saturating_to_i32(),
            other => panic!("SaturatingToI32 not supported for {}", other.datatype()),
        }
    }

    fn saturating_to_i64(&self) -> i64 {
        match self {
            Value::I8(val) => *val as i64,
            Value::I16(val) => *val as i64,
            Value::I32(val) => *val as i64,
            Value::I64(val) => *val,
            Value::I128(val) => match *val {
                val if val < i64::MIN as i128 => i64::MIN,
                val if val > i64::MAX as i128 => i64::MAX,
                val => val as i64,
            },
            Value::U8(val) => *val as i64,
            Value::U16(val) => *val as i64,
            Value::U32(val) => *val as i64,
            Value::U64(val) => match *val {
                val if val > i64::MAX as u64 => i64::MAX,
                val => val as i64,
            },
            Value::U128(val) => match *val {
                val if val > i64::MAX as u128 => i64::MAX,
                val => val as i64,
            },
            Value::F32(val) => {
                if val.is_nan() {
                    0
                } else if *val < i64::MIN as f32 {
                    i64::MIN
                } else if *val > i64::MAX as f32 {
                    i64::MAX
                } else {
                    *val as i64
                }
            }
            Value::F64(val) => {
                if val.is_nan() {
                    0
                } else if *val < i64::MIN as f64 {
                    i64::MIN
                } else if *val > i64::MAX as f64 {
                    i64::MAX
                } else {
                    *val as i64
                }
            }
            Value::Data(obj) => obj.saturating_to_i64(),
            other => panic!("SaturatingToI64 not supported for {}", other.datatype()),
        }
    }

    fn saturating_to_i128(&self) -> i128 {
        match self {
            Value::I8(val) => *val as i128,
            Value::I16(val) => *val as i128,
            Value::I32(val) => *val as i128,
            Value::I64(val) => *val as i128,
            Value::I128(val) => *val,
            Value::U8(val) => *val as i128,
            Value::U16(val) => *val as i128,
            Value::U32(val) => *val as i128,
            Value::U64(val) => *val as i128,
            Value::U128(val) => match *val {
                val if val > i128::MAX as u128 => i128::MAX,
                val => val as i128,
            },
            Value::F32(val) => {
                if val.is_nan() {
                    0
                } else if *val < i128::MIN as f32 {
                    i128::MIN
                } else if *val > i128::MAX as f32 {
                    i128::MAX
                } else {
                    *val as i128
                }
            }
            Value::F64(val) => {
                if val.is_nan() {
                    0
                } else if *val < i128::MIN as f64 {
                    i128::MIN
                } else if *val > i128::MAX as f64 {
                    i128::MAX
                } else {
                    *val as i128
                }
            }
            Value::Data(obj) => obj.saturating_to_i128(),
            other => panic!("SaturatingToI128 not supported for {}", other.datatype()),
        }
    }

    fn saturating_to_u8(&self) -> u8 {
        match self {
            Value::I8(val) => match *val {
                val if val < u8::MIN as i8 => u8::MIN,
                val => val as u8,
            },
            Value::I16(val) => match *val {
                val if val < u8::MIN as i16 => u8::MIN,
                val if val > u8::MAX as i16 => u8::MAX,
                val => val as u8,
            },
            Value::I32(val) => match *val {
                val if val < u8::MIN as i32 => u8::MIN,
                val if val > u8::MAX as i32 => u8::MAX,
                val => val as u8,
            },
            Value::I64(val) => match *val {
                val if val < u8::MIN as i64 => u8::MIN,
                val if val > u8::MAX as i64 => u8::MAX,
                val => val as u8,
            },
            Value::I128(val) => match *val {
                val if val < u8::MIN as i128 => u8::MIN,
                val if val > u8::MAX as i128 => u8::MAX,
                val => val as u8,
            },
            Value::U8(val) => *val,
            Value::U16(val) => match *val {
                val if val > u8::MAX as u16 => u8::MAX,
                val => val as u8,
            },
            Value::U32(val) => match *val {
                val if val > u8::MAX as u32 => u8::MAX,
                val => val as u8,
            },
            Value::U64(val) => match *val {
                val if val > u8::MAX as u64 => u8::MAX,
                val => val as u8,
            },
            Value::U128(val) => match *val {
                val if val > u8::MAX as u128 => u8::MAX,
                val => val as u8,
            },
            Value::F32(val) => {
                if val.is_nan() {
                    0
                } else if *val < u8::MIN as f32 {
                    u8::MIN
                } else if *val > u8::MAX as f32 {
                    u8::MAX
                } else {
                    *val as u8
                }
            }
            Value::F64(val) => {
                if val.is_nan() {
                    0
                } else if *val < u8::MIN as f64 {
                    u8::MIN
                } else if *val > u8::MAX as f64 {
                    u8::MAX
                } else {
                    *val as u8
                }
            }
            Value::Data(obj) => obj.saturating_to_u8(),
            other => panic!("SaturatingToU8 not supported for {}", other.datatype()),
        }
    }

    fn saturating_to_u16(&self) -> u16 {
        match self {
            Value::I8(val) => match *val {
                val if val < u16::MIN as i8 => u16::MIN,
                val => val as u16,
            },
            Value::I16(val) => match *val {
                val if val < u16::MIN as i16 => u16::MIN,
                val => val as u16,
            },
            Value::I32(val) => match *val {
                val if val < u16::MIN as i32 => u16::MIN,
                val if val > u16::MAX as i32 => u16::MAX,
                val => val as u16,
            },
            Value::I64(val) => match *val {
                val if val < u16::MIN as i64 => u16::MIN,
                val if val > u16::MAX as i64 => u16::MAX,
                val => val as u16,
            },
            Value::I128(val) => match *val {
                val if val < u16::MIN as i128 => u16::MIN,
                val if val > u16::MAX as i128 => u16::MAX,
                val => val as u16,
            },
            Value::U8(val) => *val as u16,
            Value::U16(val) => *val,
            Value::U32(val) => match *val {
                val if val > u16::MAX as u32 => u16::MAX,
                val => val as u16,
            },
            Value::U64(val) => match *val {
                val if val > u16::MAX as u64 => u16::MAX,
                val => val as u16,
            },
            Value::U128(val) => match *val {
                val if val > u16::MAX as u128 => u16::MAX,
                val => val as u16,
            },
            Value::F32(val) => {
                if val.is_nan() {
                    0
                } else if *val < u16::MIN as f32 {
                    u16::MIN
                } else if *val > u16::MAX as f32 {
                    u16::MAX
                } else {
                    *val as u16
                }
            }
            Value::F64(val) => {
                if val.is_nan() {
                    0
                } else if *val < u16::MIN as f64 {
                    u16::MIN
                } else if *val > u16::MAX as f64 {
                    u16::MAX
                } else {
                    *val as u16
                }
            }
            Value::Data(obj) => obj.saturating_to_u16(),
            other => panic!("SaturatingToU16 not supported for {}", other.datatype()),
        }
    }

    fn saturating_to_u32(&self) -> u32 {
        match self {
            Value::I8(val) => match *val {
                val if val < u32::MIN as i8 => u32::MIN,
                val => val as u32,
            },
            Value::I16(val) => match *val {
                val if val < u32::MIN as i16 => u32::MIN,
                val => val as u32,
            },
            Value::I32(val) => match *val {
                val if val < u32::MIN as i32 => u32::MIN,
                val => val as u32,
            },
            Value::I64(val) => match *val {
                val if val < u32::MIN as i64 => u32::MIN,
                val if val > u32::MAX as i64 => u32::MAX,
                val => val as u32,
            },
            Value::I128(val) => match *val {
                val if val < u32::MIN as i128 => u32::MIN,
                val if val > u32::MAX as i128 => u32::MAX,
                val => val as u32,
            },
            Value::U8(val) => *val as u32,
            Value::U16(val) => *val as u32,
            Value::U32(val) => *val,
            Value::U64(val) => match *val {
                val if val > u32::MAX as u64 => u32::MAX,
                val => val as u32,
            },
            Value::U128(val) => match *val {
                val if val > u32::MAX as u128 => u32::MAX,
                val => val as u32,
            },
            Value::F32(val) => {
                if val.is_nan() {
                    0
                } else if *val < u32::MIN as f32 {
                    u32::MIN
                } else if *val > u32::MAX as f32 {
                    u32::MAX
                } else {
                    *val as u32
                }
            }
            Value::F64(val) => {
                if val.is_nan() {
                    0
                } else if *val < u32::MIN as f64 {
                    u32::MIN
                } else if *val > u32::MAX as f64 {
                    u32::MAX
                } else {
                    *val as u32
                }
            }
            Value::Data(obj) => obj.saturating_to_u32(),
            other => panic!("SaturatingToU32 not supported for {}", other.datatype()),
        }
    }

    fn saturating_to_u64(&self) -> u64 {
        match self {
            Value::I8(val) => match *val {
                val if val < u64::MIN as i8 => u64::MIN,
                val => val as u64,
            },
            Value::I16(val) => match *val {
                val if val < u64::MIN as i16 => u64::MIN,
                val => val as u64,
            },
            Value::I32(val) => match *val {
                val if val < u64::MIN as i32 => u64::MIN,
                val => val as u64,
            },
            Value::I64(val) => match *val {
                val if val < u64::MIN as i64 => u64::MIN,
                val => val as u64,
            },
            Value::I128(val) => match *val {
                val if val < u64::MIN as i128 => u64::MIN,
                val if val > u64::MAX as i128 => u64::MAX,
                val => val as u64,
            },
            Value::U8(val) => *val as u64,
            Value::U16(val) => *val as u64,
            Value::U32(val) => *val as u64,
            Value::U64(val) => *val,
            Value::U128(val) => match *val {
                val if val > u64::MAX as u128 => u64::MAX,
                val => val as u64,
            },
            Value::F32(val) => {
                if val.is_nan() {
                    0
                } else if *val < u64::MIN as f32 {
                    u64::MIN
                } else if *val > u64::MAX as f32 {
                    u64::MAX
                } else {
                    *val as u64
                }
            }
            Value::F64(val) => {
                if val.is_nan() {
                    0
                } else if *val < u64::MIN as f64 {
                    u64::MIN
                } else if *val > u64::MAX as f64 {
                    u64::MAX
                } else {
                    *val as u64
                }
            }
            Value::Data(obj) => obj.saturating_to_u64(),
            other => panic!("SaturatingToU64 not supported for {}", other.datatype()),
        }
    }

    fn saturating_to_u128(&self) -> u128 {
        match self {
            Value::I8(val) => match *val {
                val if val < u128::MIN as i8 => u128::MIN,
                val => val as u128,
            },
            Value::I16(val) => match *val {
                val if val < u128::MIN as i16 => u128::MIN,
                val => val as u128,
            },
            Value::I32(val) => match *val {
                val if val < u128::MIN as i32 => u128::MIN,
                val => val as u128,
            },
            Value::I64(val) => match *val {
                val if val < u128::MIN as i64 => u128::MIN,
                val => val as u128,
            },
            Value::I128(val) => match *val {
                val if val < u128::MIN as i128 => u128::MIN,
                val => val as u128,
            },
            Value::U8(val) => *val as u128,
            Value::U16(val) => *val as u128,
            Value::U32(val) => *val as u128,
            Value::U64(val) => *val as u128,
            Value::U128(val) => *val,
            Value::F32(val) => {
                if val.is_nan() {
                    0
                } else if *val < u128::MIN as f32 {
                    u128::MIN
                } else if *val > u128::MAX as f32 {
                    u128::MAX
                } else {
                    *val as u128
                }
            }
            Value::F64(val) => {
                if val.is_nan() {
                    0
                } else if *val < u128::MIN as f64 {
                    u128::MIN
                } else if *val > u128::MAX as f64 {
                    u128::MAX
                } else {
                    *val as u128
                }
            }
            Value::Data(obj) => obj.saturating_to_u128(),
            other => panic!("SaturatingToU128 not supported for {}", other.datatype()),
        }
    }

    fn saturating_to_f32(&self) -> f32 {
        match self {
            Value::I8(val) => *val as f32,
            Value::I16(val) => *val as f32,
            Value::I32(val) => *val as f32,
            Value::I64(val) => *val as f32,
            Value::I128(val) => *val as f32,
            Value::U8(val) => *val as f32,
            Value::U16(val) => *val as f32,
            Value::U32(val) => *val as f32,
            Value::U64(val) => *val as f32,
            Value::U128(val) => *val as f32,
            Value::F32(val) => *val,
            Value::F64(val) => *val as f32,
            Value::Data(obj) => obj.saturating_to_f32(),
            other => panic!("SaturatingToF32 not supported for {}", other.datatype()),
        }
    }

    fn saturating_to_f64(&self) -> f64 {
        match self {
            Value::I8(val) => *val as f64,
            Value::I16(val) => *val as f64,
            Value::I32(val) => *val as f64,
            Value::I64(val) => *val as f64,
            Value::I128(val) => *val as f64,
            Value::U8(val) => *val as f64,
            Value::U16(val) => *val as f64,
            Value::U32(val) => *val as f64,
            Value::U64(val) => *val as f64,
            Value::U128(val) => *val as f64,
            Value::F32(val) => *val as f64,
            Value::F64(val) => *val,
            Value::Data(obj) => obj.saturating_to_f64(),
            other => panic!("SaturatingToF64 not supported for {}", other.datatype()),
        }
    }

    fn binary_and(&self, other: &Value) -> Value {
        match (self, other) {
            (Value::Bool(val), Value::Bool(other)) => Value::Bool(val & other),
            (Value::Byte(val), Value::Byte(other)) => Value::Byte(val & other),
            (Value::Data(obj), Value::Data(_)) => obj.binary_and(other),
            (a, b) if a.datatype() != b.datatype() => {
                panic!("Unsupported operation, values involved must have same type")
            }
            (other, _) => panic!("Binary not supported for {}", other.datatype()),
        }
    }

    fn binary_or(&self, other: &Value) -> Value {
        match (self, other) {
            (Value::Bool(val), Value::Bool(other)) => Value::Bool(val | other),
            (Value::Byte(val), Value::Byte(other)) => Value::Byte(val | other),
            (Value::Data(obj), Value::Data(_)) => obj.binary_or(other),
            (a, b) if a.datatype() != b.datatype() => {
                panic!("Unsupported operation, values involved must have same type")
            }
            (other, _) => panic!("Binary not supported for {}", other.datatype()),
        }
    }

    fn binary_xor(&self, other: &Value) -> Value {
        match (self, other) {
            (Value::Bool(val), Value::Bool(other)) => Value::Bool(val ^ other),
            (Value::Byte(val), Value::Byte(other)) => Value::Byte(val ^ other),
            (Value::Data(obj), Value::Data(_)) => obj.binary_xor(other),
            (a, b) if a.datatype() != b.datatype() => {
                panic!("Unsupported operation, values involved must have same type")
            }
            (other, _) => panic!("Binary not supported for {}", other.datatype()),
        }
    }

    fn binary_not(&self) -> Value {
        match self {
            Value::Bool(val) => Value::Bool(!val),
            Value::Byte(val) => Value::Byte(!val),
            Value::Data(obj) => obj.binary_not(),
            other => panic!("Binary not supported for {}", other.datatype()),
        }
    }

    fn signed_abs(&self) -> Option<Value> {
        match self {
            Value::I8(val) => {
                if *val == i8::MIN {
                    None
                } else {
                    Some(Value::I8(val.abs()))
                }
            }
            Value::I16(val) => {
                if *val == i16::MIN {
                    None
                } else {
                    Some(Value::I16(val.abs()))
                }
            }
            Value::I32(val) => {
                if *val == i32::MIN {
                    None
                } else {
                    Some(Value::I32(val.abs()))
                }
            }
            Value::I64(val) => {
                if *val == i64::MIN {
                    None
                } else {
                    Some(Value::I64(val.abs()))
                }
            }
            Value::I128(val) => {
                if *val == i128::MIN {
                    None
                } else {
                    Some(Value::I128(val.abs()))
                }
            }
            Value::F32(val) => Some(Value::F32(val.abs())),
            Value::F64(val) => Some(Value::F64(val.abs())),
            Value::Data(obj) => obj.signed_abs(),
            other => panic!("Signed not supported for {}", other.datatype()),
        }
    }

    fn signed_signum(&self) -> Value {
        match self {
            Value::I8(val) => Value::I8(val.signum()),
            Value::I16(val) => Value::I16(val.signum()),
            Value::I32(val) => Value::I32(val.signum()),
            Value::I64(val) => Value::I64(val.signum()),
            Value::I128(val) => Value::I128(val.signum()),
            Value::F32(val) => Value::F32(val.signum()),
            Value::F64(val) => Value::F64(val.signum()),
            Value::Data(obj) => obj.signed_signum(),
            other => panic!("Signed not supported for {}", other.datatype()),
        }
    }

    fn signed_is_positive(&self) -> bool {
        match self {
            Value::I8(val) => val.is_positive(),
            Value::I16(val) => val.is_positive(),
            Value::I32(val) => val.is_positive(),
            Value::I64(val) => val.is_positive(),
            Value::I128(val) => val.is_positive(),
            Value::F32(val) => val.is_sign_positive(),
            Value::F64(val) => val.is_sign_positive(),
            Value::Data(obj) => obj.signed_is_positive(),
            other => panic!("Signed not supported for {}", other.datatype()),
        }
    }

    fn signed_is_negative(&self) -> bool {
        match self {
            Value::I8(val) => val.is_negative(),
            Value::I16(val) => val.is_negative(),
            Value::I32(val) => val.is_negative(),
            Value::I64(val) => val.is_negative(),
            Value::I128(val) => val.is_negative(),
            Value::F32(val) => val.is_sign_negative(),
            Value::F64(val) => val.is_sign_negative(),
            Value::Data(obj) => obj.signed_is_negative(),
            other => panic!("Signed not supported for {}", other.datatype()),
        }
    }

    fn float_is_nan(&self) -> bool {
        match self {
            Value::F32(val) => val.is_nan(),
            Value::F64(val) => val.is_nan(),
            Value::Data(obj) => obj.float_is_nan(),
            other => panic!("Float not supported for {}", other.datatype()),
        }
    }

    fn float_is_infinite(&self) -> bool {
        match self {
            Value::F32(val) => val.is_infinite(),
            Value::F64(val) => val.is_infinite(),
            Value::Data(obj) => obj.float_is_infinite(),
            other => panic!("Float not supported for {}", other.datatype()),
        }
    }

    fn float_is_finite(&self) -> bool {
        match self {
            Value::F32(val) => val.is_finite(),
            Value::F64(val) => val.is_finite(),
            Value::Data(obj) => obj.float_is_finite(),
            other => panic!("Float not supported for {}", other.datatype()),
        }
    }

    fn float_is_normal(&self) -> bool {
        match self {
            Value::F32(val) => val.is_normal(),
            Value::F64(val) => val.is_normal(),
            Value::Data(obj) => obj.float_is_normal(),
            other => panic!("Float not supported for {}", other.datatype()),
        }
    }

    fn float_is_subnormal(&self) -> bool {
        match self {
            Value::F32(val) => val.is_subnormal(),
            Value::F64(val) => val.is_subnormal(),
            Value::Data(obj) => obj.float_is_subnormal(),
            other => panic!("Float not supported for {}", other.datatype()),
        }
    }

    fn float_floor(&self) -> Value {
        match self {
            Value::F32(val) => Value::F32(val.floor()),
            Value::F64(val) => Value::F64(val.floor()),
            Value::Data(obj) => obj.float_floor(),
            other => panic!("Float not supported for {}", other.datatype()),
        }
    }

    fn float_ceil(&self) -> Value {
        match self {
            Value::F32(val) => Value::F32(val.ceil()),
            Value::F64(val) => Value::F64(val.ceil()),
            Value::Data(obj) => obj.float_ceil(),
            other => panic!("Float not supported for {}", other.datatype()),
        }
    }

    fn float_round(&self) -> Value {
        match self {
            Value::F32(val) => Value::F32(val.round()),
            Value::F64(val) => Value::F64(val.round()),
            Value::Data(obj) => obj.float_round(),
            other => panic!("Float not supported for {}", other.datatype()),
        }
    }

    fn float_trunc(&self) -> Value {
        match self {
            Value::F32(val) => Value::F32(val.trunc()),
            Value::F64(val) => Value::F64(val.trunc()),
            Value::Data(obj) => obj.float_trunc(),
            other => panic!("Float not supported for {}", other.datatype()),
        }
    }

    fn float_fract(&self) -> Value {
        match self {
            Value::F32(val) => Value::F32(val.fract()),
            Value::F64(val) => Value::F64(val.fract()),
            Value::Data(obj) => obj.float_fract(),
            other => panic!("Float not supported for {}", other.datatype()),
        }
    }

    fn float_recip(&self) -> Value {
        match self {
            Value::F32(val) => Value::F32(val.recip()),
            Value::F64(val) => Value::F64(val.recip()),
            Value::Data(obj) => obj.float_recip(),
            other => panic!("Float not supported for {}", other.datatype()),
        }
    }

    fn float_pow(&self, n: &Value) -> Value {
        match (self, n) {
            (Value::F32(val), Value::F32(n)) => Value::F32(val.powf(*n)),
            (Value::F64(val), Value::F64(n)) => Value::F64(val.powf(*n)),
            (Value::Data(obj), Value::Data(_)) => obj.float_pow(n),
            (a, b) if a.datatype() != b.datatype() => {
                panic!("Unsupported operation, values involved must have same type")
            }
            (other, _) => panic!("Float not supported for {}", other.datatype()),
        }
    }

    fn float_sqrt(&self) -> Value {
        match self {
            Value::F32(val) => Value::F32(val.sqrt()),
            Value::F64(val) => Value::F64(val.sqrt()),
            Value::Data(obj) => obj.float_sqrt(),
            other => panic!("Float not supported for {}", other.datatype()),
        }
    }

    fn float_exp(&self) -> Value {
        match self {
            Value::F32(val) => Value::F32(val.exp()),
            Value::F64(val) => Value::F64(val.exp()),
            Value::Data(obj) => obj.float_exp(),
            other => panic!("Float not supported for {}", other.datatype()),
        }
    }

    fn float_exp2(&self) -> Value {
        match self {
            Value::F32(val) => Value::F32(val.exp2()),
            Value::F64(val) => Value::F64(val.exp2()),
            Value::Data(obj) => obj.float_exp2(),
            other => panic!("Float not supported for {}", other.datatype()),
        }
    }

    fn float_ln(&self) -> Value {
        match self {
            Value::F32(val) => Value::F32(val.ln()),
            Value::F64(val) => Value::F64(val.ln()),
            Value::Data(obj) => obj.float_ln(),
            other => panic!("Float not supported for {}", other.datatype()),
        }
    }

    fn float_log(&self, base: &Value) -> Value {
        match (self, base) {
            (Value::F32(val), Value::F32(n)) => Value::F32(val.log(*n)),
            (Value::F64(val), Value::F64(n)) => Value::F64(val.log(*n)),
            (Value::Data(obj), Value::Data(_)) => obj.float_log(base),
            (a, b) if a.datatype() != b.datatype() => {
                panic!("Unsupported operation, values involved must have same type")
            }
            (other, _) => panic!("Float not supported for {}", other.datatype()),
        }
    }

    fn float_log2(&self) -> Value {
        match self {
            Value::F32(val) => Value::F32(val.log2()),
            Value::F64(val) => Value::F64(val.log2()),
            Value::Data(obj) => obj.float_log2(),
            other => panic!("Float not supported for {}", other.datatype()),
        }
    }

    fn float_log10(&self) -> Value {
        match self {
            Value::F32(val) => Value::F32(val.log10()),
            Value::F64(val) => Value::F64(val.log10()),
            Value::Data(obj) => obj.float_log10(),
            other => panic!("Float not supported for {}", other.datatype()),
        }
    }

    fn float_cbrt(&self) -> Value {
        match self {
            Value::F32(val) => Value::F32(val.cbrt()),
            Value::F64(val) => Value::F64(val.cbrt()),
            Value::Data(obj) => obj.float_cbrt(),
            other => panic!("Float not supported for {}", other.datatype()),
        }
    }

    fn float_hypot(&self, n: &Value) -> Value {
        match (self, n) {
            (Value::F32(val), Value::F32(n)) => Value::F32(val.hypot(*n)),
            (Value::F64(val), Value::F64(n)) => Value::F64(val.hypot(*n)),
            (Value::Data(obj), Value::Data(_)) => obj.float_hypot(n),
            (a, b) if a.datatype() != b.datatype() => {
                panic!("Unsupported operation, values involved must have same type")
            }
            (other, _) => panic!("Float not supported for {}", other.datatype()),
        }
    }

    fn float_sin(&self) -> Value {
        match self {
            Value::F32(val) => Value::F32(val.sin()),
            Value::F64(val) => Value::F64(val.sin()),
            Value::Data(obj) => obj.float_sin(),
            other => panic!("Float not supported for {}", other.datatype()),
        }
    }

    fn float_cos(&self) -> Value {
        match self {
            Value::F32(val) => Value::F32(val.cos()),
            Value::F64(val) => Value::F64(val.cos()),
            Value::Data(obj) => obj.float_cos(),
            other => panic!("Float not supported for {}", other.datatype()),
        }
    }

    fn float_tan(&self) -> Value {
        match self {
            Value::F32(val) => Value::F32(val.tan()),
            Value::F64(val) => Value::F64(val.tan()),
            Value::Data(obj) => obj.float_tan(),
            other => panic!("Float not supported for {}", other.datatype()),
        }
    }

    fn float_asin(&self) -> Value {
        match self {
            Value::F32(val) => Value::F32(val.asin()),
            Value::F64(val) => Value::F64(val.asin()),
            Value::Data(obj) => obj.float_asin(),
            other => panic!("Float not supported for {}", other.datatype()),
        }
    }

    fn float_acos(&self) -> Value {
        match self {
            Value::F32(val) => Value::F32(val.acos()),
            Value::F64(val) => Value::F64(val.acos()),
            Value::Data(obj) => obj.float_acos(),
            other => panic!("Float not supported for {}", other.datatype()),
        }
    }

    fn float_atan(&self) -> Value {
        match self {
            Value::F32(val) => Value::F32(val.atan()),
            Value::F64(val) => Value::F64(val.atan()),
            Value::Data(obj) => obj.float_atan(),
            other => panic!("Float not supported for {}", other.datatype()),
        }
    }

    fn float_atan2(&self, n: &Value) -> Value {
        match (self, n) {
            (Value::F32(val), Value::F32(n)) => Value::F32(val.atan2(*n)),
            (Value::F64(val), Value::F64(n)) => Value::F64(val.atan2(*n)),
            (Value::Data(obj), Value::Data(_)) => obj.binary_and(n),
            (a, b) if a.datatype() != b.datatype() => {
                panic!("Unsupported operation, values involved must have same type")
            }
            (other, _) => panic!("Float not supported for {}", other.datatype()),
        }
    }

    fn float_sinh(&self) -> Value {
        match self {
            Value::F32(val) => Value::F32(val.sinh()),
            Value::F64(val) => Value::F64(val.sinh()),
            Value::Data(obj) => obj.float_sinh(),
            other => panic!("Float not supported for {}", other.datatype()),
        }
    }

    fn float_cosh(&self) -> Value {
        match self {
            Value::F32(val) => Value::F32(val.cosh()),
            Value::F64(val) => Value::F64(val.cosh()),
            Value::Data(obj) => obj.float_cosh(),
            other => panic!("Float not supported for {}", other.datatype()),
        }
    }

    fn float_tanh(&self) -> Value {
        match self {
            Value::F32(val) => Value::F32(val.tanh()),
            Value::F64(val) => Value::F64(val.tanh()),
            Value::Data(obj) => obj.float_tanh(),
            other => panic!("Float not supported for {}", other.datatype()),
        }
    }

    fn float_asinh(&self) -> Value {
        match self {
            Value::F32(val) => Value::F32(val.asinh()),
            Value::F64(val) => Value::F64(val.asinh()),
            Value::Data(obj) => obj.float_asinh(),
            other => panic!("Float not supported for {}", other.datatype()),
        }
    }

    fn float_acosh(&self) -> Value {
        match self {
            Value::F32(val) => Value::F32(val.acosh()),
            Value::F64(val) => Value::F64(val.acosh()),
            Value::Data(obj) => obj.float_acosh(),
            other => panic!("Float not supported for {}", other.datatype()),
        }
    }

    fn float_atanh(&self) -> Value {
        match self {
            Value::F32(val) => Value::F32(val.atanh()),
            Value::F64(val) => Value::F64(val.atanh()),
            Value::Data(obj) => obj.float_atanh(),
            other => panic!("Float not supported for {}", other.datatype()),
        }
    }

    fn float_to_degrees(&self) -> Value {
        match self {
            Value::F32(val) => Value::F32(val.to_degrees()),
            Value::F64(val) => Value::F64(val.to_degrees()),
            Value::Data(obj) => obj.float_to_degrees(),
            other => panic!("Float not supported for {}", other.datatype()),
        }
    }

    fn float_to_radians(&self) -> Value {
        match self {
            Value::F32(val) => Value::F32(val.to_radians()),
            Value::F64(val) => Value::F64(val.to_radians()),
            Value::Data(obj) => obj.float_to_radians(),
            other => panic!("Float not supported for {}", other.datatype()),
        }
    }

    fn partial_equality_eq(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::I8(me), Value::I8(other)) => me == other,
            (Value::I16(me), Value::I16(other)) => me == other,
            (Value::I32(me), Value::I32(other)) => me == other,
            (Value::I64(me), Value::I64(other)) => me == other,
            (Value::I128(me), Value::I128(other)) => me == other,
            (Value::U8(me), Value::U8(other)) => me == other,
            (Value::U16(me), Value::U16(other)) => me == other,
            (Value::U32(me), Value::U32(other)) => me == other,
            (Value::U64(me), Value::U64(other)) => me == other,
            (Value::U128(me), Value::U128(other)) => me == other,
            (Value::F32(me), Value::F32(other)) => me == other,
            (Value::F64(me), Value::F64(other)) => me == other,
            (Value::Bool(me), Value::Bool(other)) => me == other,
            (Value::Byte(me), Value::Byte(other)) => me == other,
            (Value::Char(me), Value::Char(other)) => me == other,
            (Value::String(me), Value::String(other)) => me == other,
            (Value::Data(obj), Value::Data(_)) => obj.partial_equality_eq(other),
            (a, b) if a.datatype() != b.datatype() => {
                panic!("Unsupported operation, values involved must have same type")
            }
            (other, _) => panic!("PartialEq not supported for {}", other.datatype()),
        }
    }

    fn partial_equality_ne(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::I8(me), Value::I8(other)) => me != other,
            (Value::I16(me), Value::I16(other)) => me != other,
            (Value::I32(me), Value::I32(other)) => me != other,
            (Value::I64(me), Value::I64(other)) => me != other,
            (Value::I128(me), Value::I128(other)) => me != other,
            (Value::U8(me), Value::U8(other)) => me != other,
            (Value::U16(me), Value::U16(other)) => me != other,
            (Value::U32(me), Value::U32(other)) => me != other,
            (Value::U64(me), Value::U64(other)) => me != other,
            (Value::U128(me), Value::U128(other)) => me != other,
            (Value::F32(me), Value::F32(other)) => me != other,
            (Value::F64(me), Value::F64(other)) => me != other,
            (Value::Bool(me), Value::Bool(other)) => me != other,
            (Value::Byte(me), Value::Byte(other)) => me != other,
            (Value::Char(me), Value::Char(other)) => me != other,
            (Value::String(me), Value::String(other)) => me != other,
            (Value::Data(obj), Value::Data(_)) => obj.partial_equality_ne(other),
            (a, b) if a.datatype() != b.datatype() => {
                panic!("Unsupported operation, values involved must have same type")
            }
            (other, _) => panic!("PartialEq not supported for {}", other.datatype()),
        }
    }

    fn partial_order_lt(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::I8(me), Value::I8(other)) => me < other,
            (Value::I16(me), Value::I16(other)) => me < other,
            (Value::I32(me), Value::I32(other)) => me < other,
            (Value::I64(me), Value::I64(other)) => me < other,
            (Value::I128(me), Value::I128(other)) => me < other,
            (Value::U8(me), Value::U8(other)) => me < other,
            (Value::U16(me), Value::U16(other)) => me < other,
            (Value::U32(me), Value::U32(other)) => me < other,
            (Value::U64(me), Value::U64(other)) => me < other,
            (Value::U128(me), Value::U128(other)) => me < other,
            (Value::F32(me), Value::F32(other)) => me < other,
            (Value::F64(me), Value::F64(other)) => me < other,
            (Value::Bool(me), Value::Bool(other)) => me < other,
            (Value::Byte(me), Value::Byte(other)) => me < other,
            (Value::Char(me), Value::Char(other)) => me < other,
            (Value::String(me), Value::String(other)) => me < other,
            (Value::Data(obj), Value::Data(_)) => obj.partial_order_lt(other),
            (a, b) if a.datatype() != b.datatype() => {
                panic!("Unsupported operation, values involved must have same type")
            }
            (other, _) => panic!("PartialOrd not supported for {}", other.datatype()),
        }
    }

    fn partial_order_le(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::I8(me), Value::I8(other)) => me <= other,
            (Value::I16(me), Value::I16(other)) => me <= other,
            (Value::I32(me), Value::I32(other)) => me <= other,
            (Value::I64(me), Value::I64(other)) => me <= other,
            (Value::I128(me), Value::I128(other)) => me <= other,
            (Value::U8(me), Value::U8(other)) => me <= other,
            (Value::U16(me), Value::U16(other)) => me <= other,
            (Value::U32(me), Value::U32(other)) => me <= other,
            (Value::U64(me), Value::U64(other)) => me <= other,
            (Value::U128(me), Value::U128(other)) => me <= other,
            (Value::F32(me), Value::F32(other)) => me <= other,
            (Value::F64(me), Value::F64(other)) => me <= other,
            (Value::Bool(me), Value::Bool(other)) => me <= other,
            (Value::Byte(me), Value::Byte(other)) => me <= other,
            (Value::Char(me), Value::Char(other)) => me <= other,
            (Value::String(me), Value::String(other)) => me <= other,
            (Value::Data(obj), Value::Data(_)) => obj.partial_order_le(other),
            (a, b) if a.datatype() != b.datatype() => {
                panic!("Unsupported operation, values involved must have same type")
            }
            (other, _) => panic!("PartialOrd not supported for {}", other.datatype()),
        }
    }

    fn partial_order_gt(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::I8(me), Value::I8(other)) => me > other,
            (Value::I16(me), Value::I16(other)) => me > other,
            (Value::I32(me), Value::I32(other)) => me > other,
            (Value::I64(me), Value::I64(other)) => me > other,
            (Value::I128(me), Value::I128(other)) => me > other,
            (Value::U8(me), Value::U8(other)) => me > other,
            (Value::U16(me), Value::U16(other)) => me > other,
            (Value::U32(me), Value::U32(other)) => me > other,
            (Value::U64(me), Value::U64(other)) => me > other,
            (Value::U128(me), Value::U128(other)) => me > other,
            (Value::F32(me), Value::F32(other)) => me > other,
            (Value::F64(me), Value::F64(other)) => me > other,
            (Value::Bool(me), Value::Bool(other)) => me > other,
            (Value::Byte(me), Value::Byte(other)) => me > other,
            (Value::Char(me), Value::Char(other)) => me > other,
            (Value::String(me), Value::String(other)) => me > other,
            (Value::Data(obj), Value::Data(_)) => obj.partial_order_gt(other),
            (a, b) if a.datatype() != b.datatype() => {
                panic!("Unsupported operation, values involved must have same type")
            }
            (other, _) => panic!("PartialOrd not supported for {}", other.datatype()),
        }
    }

    fn partial_order_ge(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::I8(me), Value::I8(other)) => me >= other,
            (Value::I16(me), Value::I16(other)) => me >= other,
            (Value::I32(me), Value::I32(other)) => me >= other,
            (Value::I64(me), Value::I64(other)) => me >= other,
            (Value::I128(me), Value::I128(other)) => me >= other,
            (Value::U8(me), Value::U8(other)) => me >= other,
            (Value::U16(me), Value::U16(other)) => me >= other,
            (Value::U32(me), Value::U32(other)) => me >= other,
            (Value::U64(me), Value::U64(other)) => me >= other,
            (Value::U128(me), Value::U128(other)) => me >= other,
            (Value::F32(me), Value::F32(other)) => me >= other,
            (Value::F64(me), Value::F64(other)) => me >= other,
            (Value::Bool(me), Value::Bool(other)) => me >= other,
            (Value::Byte(me), Value::Byte(other)) => me >= other,
            (Value::Char(me), Value::Char(other)) => me >= other,
            (Value::String(me), Value::String(other)) => me >= other,
            (Value::Data(obj), Value::Data(_)) => obj.partial_order_ge(other),
            (a, b) if a.datatype() != b.datatype() => {
                panic!("Unsupported operation, values involved must have same type")
            }
            (other, _) => panic!("PartialOrd not supported for {}", other.datatype()),
        }
    }

    fn order_max(&self, other: &Value) -> Value {
        match (self, other) {
            (Value::I8(me), Value::I8(other)) => Value::I8((*me).max(*other)),
            (Value::I16(me), Value::I16(other)) => Value::I16((*me).max(*other)),
            (Value::I32(me), Value::I32(other)) => Value::I32((*me).max(*other)),
            (Value::I64(me), Value::I64(other)) => Value::I64((*me).max(*other)),
            (Value::I128(me), Value::I128(other)) => Value::I128((*me).max(*other)),
            (Value::U8(me), Value::U8(other)) => Value::U8((*me).max(*other)),
            (Value::U16(me), Value::U16(other)) => Value::U16((*me).max(*other)),
            (Value::U32(me), Value::U32(other)) => Value::U32((*me).max(*other)),
            (Value::U64(me), Value::U64(other)) => Value::U64((*me).max(*other)),
            (Value::U128(me), Value::U128(other)) => Value::U128((*me).max(*other)),
            (Value::F32(me), Value::F32(other)) => Value::F32((*me).max(*other)),
            (Value::F64(me), Value::F64(other)) => Value::F64((*me).max(*other)),
            (Value::Bool(me), Value::Bool(other)) => Value::Bool((*me).max(*other)),
            (Value::Byte(me), Value::Byte(other)) => Value::Byte((*me).max(*other)),
            (Value::Char(me), Value::Char(other)) => Value::Char((*me).max(*other)),
            (Value::String(me), Value::String(other)) => Value::String(me.max(other).clone()),
            (Value::Data(obj), Value::Data(_)) => obj.order_max(other),
            (a, b) if a.datatype() != b.datatype() => {
                panic!("Unsupported operation, values involved must have same type")
            }
            (other, _) => panic!("Ord not supported for {}", other.datatype()),
        }
    }

    fn order_min(&self, other: &Value) -> Value {
        match (self, other) {
            (Value::I8(me), Value::I8(other)) => Value::I8((*me).min(*other)),
            (Value::I16(me), Value::I16(other)) => Value::I16((*me).min(*other)),
            (Value::I32(me), Value::I32(other)) => Value::I32((*me).min(*other)),
            (Value::I64(me), Value::I64(other)) => Value::I64((*me).min(*other)),
            (Value::I128(me), Value::I128(other)) => Value::I128((*me).min(*other)),
            (Value::U8(me), Value::U8(other)) => Value::U8((*me).min(*other)),
            (Value::U16(me), Value::U16(other)) => Value::U16((*me).min(*other)),
            (Value::U32(me), Value::U32(other)) => Value::U32((*me).min(*other)),
            (Value::U64(me), Value::U64(other)) => Value::U64((*me).min(*other)),
            (Value::U128(me), Value::U128(other)) => Value::U128((*me).min(*other)),
            (Value::F32(me), Value::F32(other)) => Value::F32((*me).min(*other)),
            (Value::F64(me), Value::F64(other)) => Value::F64((*me).min(*other)),
            (Value::Bool(me), Value::Bool(other)) => Value::Bool((*me).min(*other)),
            (Value::Byte(me), Value::Byte(other)) => Value::Byte((*me).min(*other)),
            (Value::Char(me), Value::Char(other)) => Value::Char((*me).min(*other)),
            (Value::String(me), Value::String(other)) => Value::String(me.min(other).clone()),
            (Value::Data(obj), Value::Data(_)) => obj.order_min(other),
            (a, b) if a.datatype() != b.datatype() => {
                panic!("Unsupported operation, values involved must have same type")
            }
            (other, _) => panic!("Ord not supported for {}", other.datatype()),
        }
    }

    fn order_clamp(&self, min: &Value, max: &Value) -> Value {
        match (self, min, max) {
            (Value::I8(me), Value::I8(min), Value::I8(max)) => Value::I8((*me).clamp(*min, *max)),
            (Value::I16(me), Value::I16(min), Value::I16(max)) => {
                Value::I16((*me).clamp(*min, *max))
            }
            (Value::I32(me), Value::I32(min), Value::I32(max)) => {
                Value::I32((*me).clamp(*min, *max))
            }
            (Value::I64(me), Value::I64(min), Value::I64(max)) => {
                Value::I64((*me).clamp(*min, *max))
            }
            (Value::I128(me), Value::I128(min), Value::I128(max)) => {
                Value::I128((*me).clamp(*min, *max))
            }
            (Value::U8(me), Value::U8(min), Value::U8(max)) => Value::U8((*me).clamp(*min, *max)),
            (Value::U16(me), Value::U16(min), Value::U16(max)) => {
                Value::U16((*me).clamp(*min, *max))
            }
            (Value::U32(me), Value::U32(min), Value::U32(max)) => {
                Value::U32((*me).clamp(*min, *max))
            }
            (Value::U64(me), Value::U64(min), Value::U64(max)) => {
                Value::U64((*me).clamp(*min, *max))
            }
            (Value::U128(me), Value::U128(min), Value::U128(max)) => {
                Value::U128((*me).clamp(*min, *max))
            }
            (Value::F32(me), Value::F32(min), Value::F32(max)) => {
                Value::F32((*me).clamp(*min, *max))
            }
            (Value::F64(me), Value::F64(min), Value::F64(max)) => {
                Value::F64((*me).clamp(*min, *max))
            }
            (Value::Bool(me), Value::Bool(min), Value::Bool(max)) => {
                Value::Bool((*me).clamp(*min, *max))
            }
            (Value::Byte(me), Value::Byte(min), Value::Byte(max)) => {
                Value::Byte((*me).clamp(*min, *max))
            }
            (Value::Char(me), Value::Char(min), Value::Char(max)) => {
                Value::Char((*me).clamp(*min, *max))
            }
            (Value::String(me), Value::String(min), Value::String(max)) => {
                Value::String((me).clamp(min, max).clone())
            }
            (Value::Data(obj), Value::Data(_), Value::Data(_)) => obj.order_clamp(min, max),
            (a, b, c) if a.datatype() != b.datatype() || a.datatype() != c.datatype() => {
                panic!("Unsupported operation, values involved must have same type")
            }
            (other, _, _) => panic!("Ord not supported for {}", other.datatype()),
        }
    }

    fn add(&self, other: &Value) -> Value {
        match (self, other) {
            (Value::I8(me), Value::I8(other)) => Value::I8(*me + *other),
            (Value::I16(me), Value::I16(other)) => Value::I16(*me + *other),
            (Value::I32(me), Value::I32(other)) => Value::I32(*me + *other),
            (Value::I64(me), Value::I64(other)) => Value::I64(*me + *other),
            (Value::I128(me), Value::I128(other)) => Value::I128(*me + *other),
            (Value::U8(me), Value::U8(other)) => Value::U8(*me + *other),
            (Value::U16(me), Value::U16(other)) => Value::U16(*me + *other),
            (Value::U32(me), Value::U32(other)) => Value::U32(*me + *other),
            (Value::U64(me), Value::U64(other)) => Value::U64(*me + *other),
            (Value::U128(me), Value::U128(other)) => Value::U128(*me + *other),
            (Value::F32(me), Value::F32(other)) => Value::F32(*me + *other),
            (Value::F64(me), Value::F64(other)) => Value::F64(*me + *other),
            (Value::Data(obj), Value::Data(_)) => obj.add(other),
            (a, b) if a.datatype() != b.datatype() => {
                panic!("Unsupported operation, values involved must have same type")
            }
            (other, _) => panic!("Add not supported for {}", other.datatype()),
        }
    }

    fn checked_add(&self, other: &Value) -> Option<Value> {
        match (self, other) {
            (Value::I8(me), Value::I8(other)) => me.checked_add(*other).map(|val| Value::I8(val)),
            (Value::I16(me), Value::I16(other)) => {
                me.checked_add(*other).map(|val| Value::I16(val))
            }
            (Value::I32(me), Value::I32(other)) => {
                me.checked_add(*other).map(|val| Value::I32(val))
            }
            (Value::I64(me), Value::I64(other)) => {
                me.checked_add(*other).map(|val| Value::I64(val))
            }
            (Value::I128(me), Value::I128(other)) => {
                me.checked_add(*other).map(|val| Value::I128(val))
            }
            (Value::U8(me), Value::U8(other)) => me.checked_add(*other).map(|val| Value::U8(val)),
            (Value::U16(me), Value::U16(other)) => {
                me.checked_add(*other).map(|val| Value::U16(val))
            }
            (Value::U32(me), Value::U32(other)) => {
                me.checked_add(*other).map(|val| Value::U32(val))
            }
            (Value::U64(me), Value::U64(other)) => {
                me.checked_add(*other).map(|val| Value::U64(val))
            }
            (Value::U128(me), Value::U128(other)) => {
                me.checked_add(*other).map(|val| Value::U128(val))
            }
            (Value::Data(obj), Value::Data(_)) => obj.checked_add(other),
            (a, b) if a.datatype() != b.datatype() => {
                panic!("Unsupported operation, values involved must have same type")
            }
            (other, _) => panic!("CheckedAdd not supported for {}", other.datatype()),
        }
    }

    fn saturating_add(&self, other: &Value) -> Value {
        match (self, other) {
            (Value::I8(me), Value::I8(other)) => Value::I8(me.saturating_add(*other)),
            (Value::I16(me), Value::I16(other)) => Value::I16(me.saturating_add(*other)),
            (Value::I32(me), Value::I32(other)) => Value::I32(me.saturating_add(*other)),
            (Value::I64(me), Value::I64(other)) => Value::I64(me.saturating_add(*other)),
            (Value::I128(me), Value::I128(other)) => Value::I128(me.saturating_add(*other)),
            (Value::U8(me), Value::U8(other)) => Value::U8(me.saturating_add(*other)),
            (Value::U16(me), Value::U16(other)) => Value::U16(me.saturating_add(*other)),
            (Value::U32(me), Value::U32(other)) => Value::U32(me.saturating_add(*other)),
            (Value::U64(me), Value::U64(other)) => Value::U64(me.saturating_add(*other)),
            (Value::U128(me), Value::U128(other)) => Value::U128(me.saturating_add(*other)),
            (Value::Data(obj), Value::Data(_)) => obj.saturating_add(other),
            (a, b) if a.datatype() != b.datatype() => {
                panic!("Unsupported operation, values involved must have same type")
            }
            (other, _) => panic!("SaturatingAdd not supported for {}", other.datatype()),
        }
    }

    fn wrapping_add(&self, other: &Value) -> Value {
        match (self, other) {
            (Value::I8(me), Value::I8(other)) => Value::I8(me.wrapping_add(*other)),
            (Value::I16(me), Value::I16(other)) => Value::I16(me.wrapping_add(*other)),
            (Value::I32(me), Value::I32(other)) => Value::I32(me.wrapping_add(*other)),
            (Value::I64(me), Value::I64(other)) => Value::I64(me.wrapping_add(*other)),
            (Value::I128(me), Value::I128(other)) => Value::I128(me.wrapping_add(*other)),
            (Value::U8(me), Value::U8(other)) => Value::U8(me.wrapping_add(*other)),
            (Value::U16(me), Value::U16(other)) => Value::U16(me.wrapping_add(*other)),
            (Value::U32(me), Value::U32(other)) => Value::U32(me.wrapping_add(*other)),
            (Value::U64(me), Value::U64(other)) => Value::U64(me.wrapping_add(*other)),
            (Value::U128(me), Value::U128(other)) => Value::U128(me.wrapping_add(*other)),
            (Value::Data(obj), Value::Data(_)) => obj.wrapping_add(other),
            (a, b) if a.datatype() != b.datatype() => {
                panic!("Unsupported operation, values involved must have same type")
            }
            (other, _) => panic!("WrappingAdd not supported for {}", other.datatype()),
        }
    }

    fn sub(&self, other: &Value) -> Value {
        match (self, other) {
            (Value::I8(me), Value::I8(other)) => Value::I8(*me - *other),
            (Value::I16(me), Value::I16(other)) => Value::I16(*me - *other),
            (Value::I32(me), Value::I32(other)) => Value::I32(*me - *other),
            (Value::I64(me), Value::I64(other)) => Value::I64(*me - *other),
            (Value::I128(me), Value::I128(other)) => Value::I128(*me - *other),
            (Value::U8(me), Value::U8(other)) => Value::U8(*me - *other),
            (Value::U16(me), Value::U16(other)) => Value::U16(*me - *other),
            (Value::U32(me), Value::U32(other)) => Value::U32(*me - *other),
            (Value::U64(me), Value::U64(other)) => Value::U64(*me - *other),
            (Value::U128(me), Value::U128(other)) => Value::U128(*me - *other),
            (Value::F32(me), Value::F32(other)) => Value::F32(*me - *other),
            (Value::F64(me), Value::F64(other)) => Value::F64(*me - *other),
            (Value::Data(obj), Value::Data(_)) => obj.sub(other),
            (a, b) if a.datatype() != b.datatype() => {
                panic!("Unsupported operation, values involved must have same type")
            }
            (other, _) => panic!("Sub not supported for {}", other.datatype()),
        }
    }

    fn checked_sub(&self, other: &Value) -> Option<Value> {
        match (self, other) {
            (Value::I8(me), Value::I8(other)) => me.checked_sub(*other).map(|val| Value::I8(val)),
            (Value::I16(me), Value::I16(other)) => {
                me.checked_sub(*other).map(|val| Value::I16(val))
            }
            (Value::I32(me), Value::I32(other)) => {
                me.checked_sub(*other).map(|val| Value::I32(val))
            }
            (Value::I64(me), Value::I64(other)) => {
                me.checked_sub(*other).map(|val| Value::I64(val))
            }
            (Value::I128(me), Value::I128(other)) => {
                me.checked_sub(*other).map(|val| Value::I128(val))
            }
            (Value::U8(me), Value::U8(other)) => me.checked_sub(*other).map(|val| Value::U8(val)),
            (Value::U16(me), Value::U16(other)) => {
                me.checked_sub(*other).map(|val| Value::U16(val))
            }
            (Value::U32(me), Value::U32(other)) => {
                me.checked_sub(*other).map(|val| Value::U32(val))
            }
            (Value::U64(me), Value::U64(other)) => {
                me.checked_sub(*other).map(|val| Value::U64(val))
            }
            (Value::U128(me), Value::U128(other)) => {
                me.checked_sub(*other).map(|val| Value::U128(val))
            }
            (Value::Data(obj), Value::Data(_)) => obj.checked_sub(other),
            (a, b) if a.datatype() != b.datatype() => {
                panic!("Unsupported operation, values involved must have same type")
            }
            (other, _) => panic!("CheckedSub not supported for {}", other.datatype()),
        }
    }

    fn saturating_sub(&self, other: &Value) -> Value {
        match (self, other) {
            (Value::I8(me), Value::I8(other)) => Value::I8(me.saturating_sub(*other)),
            (Value::I16(me), Value::I16(other)) => Value::I16(me.saturating_sub(*other)),
            (Value::I32(me), Value::I32(other)) => Value::I32(me.saturating_sub(*other)),
            (Value::I64(me), Value::I64(other)) => Value::I64(me.saturating_sub(*other)),
            (Value::I128(me), Value::I128(other)) => Value::I128(me.saturating_sub(*other)),
            (Value::U8(me), Value::U8(other)) => Value::U8(me.saturating_sub(*other)),
            (Value::U16(me), Value::U16(other)) => Value::U16(me.saturating_sub(*other)),
            (Value::U32(me), Value::U32(other)) => Value::U32(me.saturating_sub(*other)),
            (Value::U64(me), Value::U64(other)) => Value::U64(me.saturating_sub(*other)),
            (Value::U128(me), Value::U128(other)) => Value::U128(me.saturating_sub(*other)),
            (Value::Data(obj), Value::Data(_)) => obj.saturating_sub(other),
            (a, b) if a.datatype() != b.datatype() => {
                panic!("Unsupported operation, values involved must have same type")
            }
            (other, _) => panic!("SaturatingSub not supported for {}", other.datatype()),
        }
    }

    fn wrapping_sub(&self, other: &Value) -> Value {
        match (self, other) {
            (Value::I8(me), Value::I8(other)) => Value::I8(me.wrapping_sub(*other)),
            (Value::I16(me), Value::I16(other)) => Value::I16(me.wrapping_sub(*other)),
            (Value::I32(me), Value::I32(other)) => Value::I32(me.wrapping_sub(*other)),
            (Value::I64(me), Value::I64(other)) => Value::I64(me.wrapping_sub(*other)),
            (Value::I128(me), Value::I128(other)) => Value::I128(me.wrapping_sub(*other)),
            (Value::U8(me), Value::U8(other)) => Value::U8(me.wrapping_sub(*other)),
            (Value::U16(me), Value::U16(other)) => Value::U16(me.wrapping_sub(*other)),
            (Value::U32(me), Value::U32(other)) => Value::U32(me.wrapping_sub(*other)),
            (Value::U64(me), Value::U64(other)) => Value::U64(me.wrapping_sub(*other)),
            (Value::U128(me), Value::U128(other)) => Value::U128(me.wrapping_sub(*other)),
            (Value::Data(obj), Value::Data(_)) => obj.wrapping_sub(other),
            (a, b) if a.datatype() != b.datatype() => {
                panic!("Unsupported operation, values involved must have same type")
            }
            (other, _) => panic!("WrappingSub not supported for {}", other.datatype()),
        }
    }

    fn mul(&self, other: &Value) -> Value {
        match (self, other) {
            (Value::I8(me), Value::I8(other)) => Value::I8(*me * *other),
            (Value::I16(me), Value::I16(other)) => Value::I16(*me * *other),
            (Value::I32(me), Value::I32(other)) => Value::I32(*me * *other),
            (Value::I64(me), Value::I64(other)) => Value::I64(*me * *other),
            (Value::I128(me), Value::I128(other)) => Value::I128(*me * *other),
            (Value::U8(me), Value::U8(other)) => Value::U8(*me * *other),
            (Value::U16(me), Value::U16(other)) => Value::U16(*me * *other),
            (Value::U32(me), Value::U32(other)) => Value::U32(*me * *other),
            (Value::U64(me), Value::U64(other)) => Value::U64(*me * *other),
            (Value::U128(me), Value::U128(other)) => Value::U128(*me * *other),
            (Value::F32(me), Value::F32(other)) => Value::F32(*me * *other),
            (Value::F64(me), Value::F64(other)) => Value::F64(*me * *other),
            (Value::Data(obj), Value::Data(_)) => obj.mul(other),
            (a, b) if a.datatype() != b.datatype() => {
                panic!("Unsupported operation, values involved must have same type")
            }
            (other, _) => panic!("Mul not supported for {}", other.datatype()),
        }
    }

    fn checked_mul(&self, other: &Value) -> Option<Value> {
        match (self, other) {
            (Value::I8(me), Value::I8(other)) => me.checked_mul(*other).map(|val| Value::I8(val)),
            (Value::I16(me), Value::I16(other)) => {
                me.checked_mul(*other).map(|val| Value::I16(val))
            }
            (Value::I32(me), Value::I32(other)) => {
                me.checked_mul(*other).map(|val| Value::I32(val))
            }
            (Value::I64(me), Value::I64(other)) => {
                me.checked_mul(*other).map(|val| Value::I64(val))
            }
            (Value::I128(me), Value::I128(other)) => {
                me.checked_mul(*other).map(|val| Value::I128(val))
            }
            (Value::U8(me), Value::U8(other)) => me.checked_mul(*other).map(|val| Value::U8(val)),
            (Value::U16(me), Value::U16(other)) => {
                me.checked_mul(*other).map(|val| Value::U16(val))
            }
            (Value::U32(me), Value::U32(other)) => {
                me.checked_mul(*other).map(|val| Value::U32(val))
            }
            (Value::U64(me), Value::U64(other)) => {
                me.checked_mul(*other).map(|val| Value::U64(val))
            }
            (Value::U128(me), Value::U128(other)) => {
                me.checked_mul(*other).map(|val| Value::U128(val))
            }
            (Value::Data(obj), Value::Data(_)) => obj.checked_mul(other),
            (a, b) if a.datatype() != b.datatype() => {
                panic!("Unsupported operation, values involved must have same type")
            }
            (other, _) => panic!("CheckedMul not supported for {}", other.datatype()),
        }
    }

    fn saturating_mul(&self, other: &Value) -> Value {
        match (self, other) {
            (Value::I8(me), Value::I8(other)) => Value::I8(me.saturating_mul(*other)),
            (Value::I16(me), Value::I16(other)) => Value::I16(me.saturating_mul(*other)),
            (Value::I32(me), Value::I32(other)) => Value::I32(me.saturating_mul(*other)),
            (Value::I64(me), Value::I64(other)) => Value::I64(me.saturating_mul(*other)),
            (Value::I128(me), Value::I128(other)) => Value::I128(me.saturating_mul(*other)),
            (Value::U8(me), Value::U8(other)) => Value::U8(me.saturating_mul(*other)),
            (Value::U16(me), Value::U16(other)) => Value::U16(me.saturating_mul(*other)),
            (Value::U32(me), Value::U32(other)) => Value::U32(me.saturating_mul(*other)),
            (Value::U64(me), Value::U64(other)) => Value::U64(me.saturating_mul(*other)),
            (Value::U128(me), Value::U128(other)) => Value::U128(me.saturating_mul(*other)),
            (Value::Data(obj), Value::Data(_)) => obj.saturating_mul(other),
            (a, b) if a.datatype() != b.datatype() => {
                panic!("Unsupported operation, values involved must have same type")
            }
            (other, _) => panic!("SaturatingMul not supported for {}", other.datatype()),
        }
    }

    fn wrapping_mul(&self, other: &Value) -> Value {
        match (self, other) {
            (Value::I8(me), Value::I8(other)) => Value::I8(me.wrapping_mul(*other)),
            (Value::I16(me), Value::I16(other)) => Value::I16(me.wrapping_mul(*other)),
            (Value::I32(me), Value::I32(other)) => Value::I32(me.wrapping_mul(*other)),
            (Value::I64(me), Value::I64(other)) => Value::I64(me.wrapping_mul(*other)),
            (Value::I128(me), Value::I128(other)) => Value::I128(me.wrapping_mul(*other)),
            (Value::U8(me), Value::U8(other)) => Value::U8(me.wrapping_mul(*other)),
            (Value::U16(me), Value::U16(other)) => Value::U16(me.wrapping_mul(*other)),
            (Value::U32(me), Value::U32(other)) => Value::U32(me.wrapping_mul(*other)),
            (Value::U64(me), Value::U64(other)) => Value::U64(me.wrapping_mul(*other)),
            (Value::U128(me), Value::U128(other)) => Value::U128(me.wrapping_mul(*other)),
            (Value::Data(obj), Value::Data(_)) => obj.wrapping_mul(other),
            (a, b) if a.datatype() != b.datatype() => {
                panic!("Unsupported operation, values involved must have same type")
            }
            (other, _) => panic!("WrappingMul not supported for {}", other.datatype()),
        }
    }

    fn div(&self, other: &Value) -> Value {
        match (self, other) {
            (Value::I8(me), Value::I8(other)) => {
                Value::I8(if *other != 0 { *me / *other } else { 0 })
            }
            (Value::I16(me), Value::I16(other)) => {
                Value::I16(if *other != 0 { *me / *other } else { 0 })
            }
            (Value::I32(me), Value::I32(other)) => {
                Value::I32(if *other != 0 { *me / *other } else { 0 })
            }
            (Value::I64(me), Value::I64(other)) => {
                Value::I64(if *other != 0 { *me / *other } else { 0 })
            }
            (Value::I128(me), Value::I128(other)) => {
                Value::I128(if *other != 0 { *me / *other } else { 0 })
            }
            (Value::U8(me), Value::U8(other)) => {
                Value::U8(if *other != 0 { *me / *other } else { 0 })
            }
            (Value::U16(me), Value::U16(other)) => {
                Value::U16(if *other != 0 { *me / *other } else { 0 })
            }
            (Value::U32(me), Value::U32(other)) => {
                Value::U32(if *other != 0 { *me / *other } else { 0 })
            }
            (Value::U64(me), Value::U64(other)) => {
                Value::U64(if *other != 0 { *me / *other } else { 0 })
            }
            (Value::U128(me), Value::U128(other)) => {
                Value::U128(if *other != 0 { *me / *other } else { 0 })
            }
            (Value::F32(me), Value::F32(other)) => Value::F32(*me / *other),
            (Value::F64(me), Value::F64(other)) => Value::F64(*me / *other),
            (Value::Data(obj), Value::Data(_)) => obj.div(other),
            (a, b) if a.datatype() != b.datatype() => {
                panic!("Unsupported operation, values involved must have same type")
            }
            (other, _) => panic!("Div not supported for {}", other.datatype()),
        }
    }

    fn checked_div(&self, other: &Value) -> Option<Value> {
        match (self, other) {
            (Value::I8(me), Value::I8(other)) => me.checked_div(*other).map(|val| Value::I8(val)),
            (Value::I16(me), Value::I16(other)) => {
                me.checked_div(*other).map(|val| Value::I16(val))
            }
            (Value::I32(me), Value::I32(other)) => {
                me.checked_div(*other).map(|val| Value::I32(val))
            }
            (Value::I64(me), Value::I64(other)) => {
                me.checked_div(*other).map(|val| Value::I64(val))
            }
            (Value::I128(me), Value::I128(other)) => {
                me.checked_div(*other).map(|val| Value::I128(val))
            }
            (Value::U8(me), Value::U8(other)) => me.checked_div(*other).map(|val| Value::U8(val)),
            (Value::U16(me), Value::U16(other)) => {
                me.checked_div(*other).map(|val| Value::U16(val))
            }
            (Value::U32(me), Value::U32(other)) => {
                me.checked_div(*other).map(|val| Value::U32(val))
            }
            (Value::U64(me), Value::U64(other)) => {
                me.checked_div(*other).map(|val| Value::U64(val))
            }
            (Value::U128(me), Value::U128(other)) => {
                me.checked_div(*other).map(|val| Value::U128(val))
            }
            (Value::Data(obj), Value::Data(_)) => obj.checked_div(other),
            (a, b) if a.datatype() != b.datatype() => {
                panic!("Unsupported operation, values involved must have same type")
            }
            (other, _) => panic!("CheckedDiv not supported for {}", other.datatype()),
        }
    }

    fn rem(&self, other: &Value) -> Value {
        match (self, other) {
            (Value::I8(me), Value::I8(other)) => {
                Value::I8(if *other != 0 { *me % *other } else { 0 })
            }
            (Value::I16(me), Value::I16(other)) => {
                Value::I16(if *other != 0 { *me % *other } else { 0 })
            }
            (Value::I32(me), Value::I32(other)) => {
                Value::I32(if *other != 0 { *me % *other } else { 0 })
            }
            (Value::I64(me), Value::I64(other)) => {
                Value::I64(if *other != 0 { *me % *other } else { 0 })
            }
            (Value::I128(me), Value::I128(other)) => {
                Value::I128(if *other != 0 { *me % *other } else { 0 })
            }
            (Value::U8(me), Value::U8(other)) => {
                Value::U8(if *other != 0 { *me % *other } else { 0 })
            }
            (Value::U16(me), Value::U16(other)) => {
                Value::U16(if *other != 0 { *me % *other } else { 0 })
            }
            (Value::U32(me), Value::U32(other)) => {
                Value::U32(if *other != 0 { *me % *other } else { 0 })
            }
            (Value::U64(me), Value::U64(other)) => {
                Value::U64(if *other != 0 { *me % *other } else { 0 })
            }
            (Value::U128(me), Value::U128(other)) => {
                Value::U128(if *other != 0 { *me % *other } else { 0 })
            }
            (Value::F32(me), Value::F32(other)) => Value::F32(*me % *other),
            (Value::F64(me), Value::F64(other)) => Value::F64(*me % *other),
            (Value::Data(obj), Value::Data(_)) => obj.rem(other),
            (a, b) if a.datatype() != b.datatype() => {
                panic!("Unsupported operation, values involved must have same type")
            }
            (other, _) => panic!("Rem not supported for {}", other.datatype()),
        }
    }

    fn checked_rem(&self, other: &Value) -> Option<Value> {
        match (self, other) {
            (Value::I8(me), Value::I8(other)) => me.checked_rem(*other).map(|val| Value::I8(val)),
            (Value::I16(me), Value::I16(other)) => {
                me.checked_rem(*other).map(|val| Value::I16(val))
            }
            (Value::I32(me), Value::I32(other)) => {
                me.checked_rem(*other).map(|val| Value::I32(val))
            }
            (Value::I64(me), Value::I64(other)) => {
                me.checked_rem(*other).map(|val| Value::I64(val))
            }
            (Value::I128(me), Value::I128(other)) => {
                me.checked_rem(*other).map(|val| Value::I128(val))
            }
            (Value::U8(me), Value::U8(other)) => me.checked_rem(*other).map(|val| Value::U8(val)),
            (Value::U16(me), Value::U16(other)) => {
                me.checked_rem(*other).map(|val| Value::U16(val))
            }
            (Value::U32(me), Value::U32(other)) => {
                me.checked_rem(*other).map(|val| Value::U32(val))
            }
            (Value::U64(me), Value::U64(other)) => {
                me.checked_rem(*other).map(|val| Value::U64(val))
            }
            (Value::U128(me), Value::U128(other)) => {
                me.checked_rem(*other).map(|val| Value::U128(val))
            }
            (Value::Data(obj), Value::Data(_)) => obj.checked_rem(other),
            (a, b) if a.datatype() != b.datatype() => {
                panic!("Unsupported operation, values involved must have same type")
            }
            (other, _) => panic!("CheckedRem not supported for {}", other.datatype()),
        }
    }

    fn neg(&self) -> Value {
        match self {
            Value::I8(me) => Value::I8(-*me),
            Value::I16(me) => Value::I16(-*me),
            Value::I32(me) => Value::I32(-*me),
            Value::I64(me) => Value::I64(-*me),
            Value::I128(me) => Value::I128(-*me),
            Value::F32(me) => Value::F32(-*me),
            Value::F64(me) => Value::F64(-*me),
            Value::Data(obj) => obj.neg(),
            other => panic!("Neg not supported for {}", other.datatype()),
        }
    }

    fn checked_neg(&self) -> Option<Value> {
        match self {
            Value::I8(me) => me.checked_neg().map(|val| Value::I8(val)),
            Value::I16(me) => me.checked_neg().map(|val| Value::I16(val)),
            Value::I32(me) => me.checked_neg().map(|val| Value::I32(val)),
            Value::I64(me) => me.checked_neg().map(|val| Value::I64(val)),
            Value::I128(me) => me.checked_neg().map(|val| Value::I128(val)),
            Value::Data(obj) => obj.checked_neg(),
            other => panic!("CheckedNeg not supported for {}", other.datatype()),
        }
    }

    fn wrapping_neg(&self) -> Value {
        match self {
            Value::I8(me) => Value::I8(me.wrapping_neg()),
            Value::I16(me) => Value::I16(me.wrapping_neg()),
            Value::I32(me) => Value::I32(me.wrapping_neg()),
            Value::I64(me) => Value::I64(me.wrapping_neg()),
            Value::I128(me) => Value::I128(me.wrapping_neg()),
            Value::Data(obj) => obj.wrapping_neg(),
            other => panic!("WrappingNeg not supported for {}", other.datatype()),
        }
    }

    fn pow(&self, exp: &u32) -> Value {
        match self {
            Value::I8(me) => Value::I8(me.pow(*exp)),
            Value::I16(me) => Value::I16(me.pow(*exp)),
            Value::I32(me) => Value::I32(me.pow(*exp)),
            Value::I64(me) => Value::I64(me.pow(*exp)),
            Value::I128(me) => Value::I128(me.pow(*exp)),
            Value::U8(me) => Value::U8(me.pow(*exp)),
            Value::U16(me) => Value::U16(me.pow(*exp)),
            Value::U32(me) => Value::U32(me.pow(*exp)),
            Value::U64(me) => Value::U64(me.pow(*exp)),
            Value::U128(me) => Value::U128(me.pow(*exp)),
            Value::Data(obj) => obj.pow(exp),
            other => panic!("Pow not supported for {}", other.datatype()),
        }
    }

    fn checked_pow(&self, exp: &u32) -> Option<Value> {
        match self {
            Value::I8(me) => me.checked_pow(*exp).map(|val| Value::I8(val)),
            Value::I16(me) => me.checked_pow(*exp).map(|val| Value::I16(val)),
            Value::I32(me) => me.checked_pow(*exp).map(|val| Value::I32(val)),
            Value::I64(me) => me.checked_pow(*exp).map(|val| Value::I64(val)),
            Value::I128(me) => me.checked_pow(*exp).map(|val| Value::I128(val)),
            Value::U8(me) => me.checked_pow(*exp).map(|val| Value::U8(val)),
            Value::U16(me) => me.checked_pow(*exp).map(|val| Value::U16(val)),
            Value::U32(me) => me.checked_pow(*exp).map(|val| Value::U32(val)),
            Value::U64(me) => me.checked_pow(*exp).map(|val| Value::U64(val)),
            Value::U128(me) => me.checked_pow(*exp).map(|val| Value::U128(val)),
            Value::Data(obj) => obj.checked_pow(exp),
            other => panic!("CheckedPow not supported for {}", other.datatype()),
        }
    }

    fn euclid_div(&self, other: &Value) -> Value {
        match (self, other) {
            (Value::I8(me), Value::I8(other)) => Value::I8(if *other != 0 {
                me.overflowing_div_euclid(*other).0
            } else {
                0
            }),
            (Value::I16(me), Value::I16(other)) => Value::I16(if *other != 0 {
                me.overflowing_div_euclid(*other).0
            } else {
                0
            }),
            (Value::I32(me), Value::I32(other)) => Value::I32(if *other != 0 {
                me.overflowing_div_euclid(*other).0
            } else {
                0
            }),
            (Value::I64(me), Value::I64(other)) => Value::I64(if *other != 0 {
                me.overflowing_div_euclid(*other).0
            } else {
                0
            }),
            (Value::I128(me), Value::I128(other)) => Value::I128(if *other != 0 {
                me.overflowing_div_euclid(*other).0
            } else {
                0
            }),
            (Value::U8(me), Value::U8(other)) => Value::U8(if *other != 0 {
                me.overflowing_div_euclid(*other).0
            } else {
                0
            }),
            (Value::U16(me), Value::U16(other)) => Value::U16(if *other != 0 {
                me.overflowing_div_euclid(*other).0
            } else {
                0
            }),
            (Value::U32(me), Value::U32(other)) => Value::U32(if *other != 0 {
                me.overflowing_div_euclid(*other).0
            } else {
                0
            }),
            (Value::U64(me), Value::U64(other)) => Value::U64(if *other != 0 {
                me.overflowing_div_euclid(*other).0
            } else {
                0
            }),
            (Value::U128(me), Value::U128(other)) => Value::U128(if *other != 0 {
                me.overflowing_div_euclid(*other).0
            } else {
                0
            }),
            (Value::F32(me), Value::F32(other)) => Value::F32(me.div_euclid(*other)),
            (Value::F64(me), Value::F64(other)) => Value::F64(me.div_euclid(*other)),
            (Value::Data(obj), Value::Data(_)) => obj.euclid_div(other),
            (a, b) if a.datatype() != b.datatype() => {
                panic!("Unsupported operation, values involved must have same type")
            }
            (other, _) => panic!("Euclid not supported for {}", other.datatype()),
        }
    }

    fn euclid_rem(&self, other: &Value) -> Value {
        match (self, other) {
            (Value::I8(me), Value::I8(other)) => Value::I8(if *other != 0 {
                me.overflowing_rem_euclid(*other).0
            } else {
                0
            }),
            (Value::I16(me), Value::I16(other)) => Value::I16(if *other != 0 {
                me.overflowing_rem_euclid(*other).0
            } else {
                0
            }),
            (Value::I32(me), Value::I32(other)) => Value::I32(if *other != 0 {
                me.overflowing_rem_euclid(*other).0
            } else {
                0
            }),
            (Value::I64(me), Value::I64(other)) => Value::I64(if *other != 0 {
                me.overflowing_rem_euclid(*other).0
            } else {
                0
            }),
            (Value::I128(me), Value::I128(other)) => Value::I128(if *other != 0 {
                me.overflowing_rem_euclid(*other).0
            } else {
                0
            }),
            (Value::U8(me), Value::U8(other)) => Value::U8(if *other != 0 {
                me.overflowing_rem_euclid(*other).0
            } else {
                0
            }),
            (Value::U16(me), Value::U16(other)) => Value::U16(if *other != 0 {
                me.overflowing_rem_euclid(*other).0
            } else {
                0
            }),
            (Value::U32(me), Value::U32(other)) => Value::U32(if *other != 0 {
                me.overflowing_rem_euclid(*other).0
            } else {
                0
            }),
            (Value::U64(me), Value::U64(other)) => Value::U64(if *other != 0 {
                me.overflowing_rem_euclid(*other).0
            } else {
                0
            }),
            (Value::U128(me), Value::U128(other)) => Value::U128(if *other != 0 {
                me.overflowing_rem_euclid(*other).0
            } else {
                0
            }),
            (Value::F32(me), Value::F32(other)) => Value::F32(me.rem_euclid(*other)),
            (Value::F64(me), Value::F64(other)) => Value::F64(me.rem_euclid(*other)),
            (Value::Data(obj), Value::Data(_)) => obj.euclid_rem(other),
            (a, b) if a.datatype() != b.datatype() => {
                panic!("Unsupported operation, values involved must have same type")
            }
            (other, _) => panic!("Euclid not supported for {}", other.datatype()),
        }
    }

    fn checked_euclid_div(&self, other: &Value) -> Option<Value> {
        match (self, other) {
            (Value::I8(me), Value::I8(other)) => {
                me.checked_div_euclid(*other).map(|val| Value::I8(val))
            }
            (Value::I16(me), Value::I16(other)) => {
                me.checked_div_euclid(*other).map(|val| Value::I16(val))
            }
            (Value::I32(me), Value::I32(other)) => {
                me.checked_div_euclid(*other).map(|val| Value::I32(val))
            }
            (Value::I64(me), Value::I64(other)) => {
                me.checked_div_euclid(*other).map(|val| Value::I64(val))
            }
            (Value::I128(me), Value::I128(other)) => {
                me.checked_div_euclid(*other).map(|val| Value::I128(val))
            }
            (Value::U8(me), Value::U8(other)) => {
                me.checked_div_euclid(*other).map(|val| Value::U8(val))
            }
            (Value::U16(me), Value::U16(other)) => {
                me.checked_div_euclid(*other).map(|val| Value::U16(val))
            }
            (Value::U32(me), Value::U32(other)) => {
                me.checked_div_euclid(*other).map(|val| Value::U32(val))
            }
            (Value::U64(me), Value::U64(other)) => {
                me.checked_div_euclid(*other).map(|val| Value::U64(val))
            }
            (Value::U128(me), Value::U128(other)) => {
                me.checked_div_euclid(*other).map(|val| Value::U128(val))
            }
            (Value::Data(obj), Value::Data(_)) => obj.checked_euclid_div(other),
            (a, b) if a.datatype() != b.datatype() => {
                panic!("Unsupported operation, values involved must have same type")
            }
            (other, _) => panic!("CheckedEuclid not supported for {}", other.datatype()),
        }
    }

    fn checked_euclid_rem(&self, other: &Value) -> Option<Value> {
        match (self, other) {
            (Value::I8(me), Value::I8(other)) => {
                me.checked_rem_euclid(*other).map(|val| Value::I8(val))
            }
            (Value::I16(me), Value::I16(other)) => {
                me.checked_rem_euclid(*other).map(|val| Value::I16(val))
            }
            (Value::I32(me), Value::I32(other)) => {
                me.checked_rem_euclid(*other).map(|val| Value::I32(val))
            }
            (Value::I64(me), Value::I64(other)) => {
                me.checked_rem_euclid(*other).map(|val| Value::I64(val))
            }
            (Value::I128(me), Value::I128(other)) => {
                me.checked_rem_euclid(*other).map(|val| Value::I128(val))
            }
            (Value::U8(me), Value::U8(other)) => {
                me.checked_rem_euclid(*other).map(|val| Value::U8(val))
            }
            (Value::U16(me), Value::U16(other)) => {
                me.checked_rem_euclid(*other).map(|val| Value::U16(val))
            }
            (Value::U32(me), Value::U32(other)) => {
                me.checked_rem_euclid(*other).map(|val| Value::U32(val))
            }
            (Value::U64(me), Value::U64(other)) => {
                me.checked_rem_euclid(*other).map(|val| Value::U64(val))
            }
            (Value::U128(me), Value::U128(other)) => {
                me.checked_rem_euclid(*other).map(|val| Value::U128(val))
            }
            (Value::Data(obj), Value::Data(_)) => obj.checked_euclid_rem(other),
            (a, b) if a.datatype() != b.datatype() => {
                panic!("Unsupported operation, values involved must have same type")
            }
            (other, _) => panic!("CheckedEuclid not supported for {}", other.datatype()),
        }
    }

    fn hash(&self, mut state: &mut dyn std::hash::Hasher) {
        match self {
            Value::I8(me) => me.hash(&mut state),
            Value::I16(me) => me.hash(&mut state),
            Value::I32(me) => me.hash(&mut state),
            Value::I64(me) => me.hash(&mut state),
            Value::I128(me) => me.hash(&mut state),
            Value::U8(me) => me.hash(&mut state),
            Value::U16(me) => me.hash(&mut state),
            Value::U32(me) => me.hash(&mut state),
            Value::U64(me) => me.hash(&mut state),
            Value::U128(me) => me.hash(&mut state),
            Value::Bool(me) => me.hash(&mut state),
            Value::Byte(me) => me.hash(&mut state),
            Value::Char(me) => me.hash(&mut state),
            Value::String(me) => me.hash(&mut state),
            Value::Vec(me) => me.iter().for_each(|elmt| elmt.hash(&mut state)),
            Value::Option(me) => {
                if let Some(elmt) = me {
                    elmt.hash(&mut state)
                } else {
                    ().hash(&mut state)
                }
            }
            Value::Data(me) => me.hash(&mut state),
            other => panic!("Hash not supported for {}", other.datatype()),
        }
    }

    fn serialize(
        &self,
        serializer: &mut dyn erased_serde::Serializer,
    ) -> Result<(), erased_serde::Error> {
        self.erased_serialize(serializer)
    }

    fn display(&self, mut f: &mut std::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        match self {
            Value::Data(data) => data.display(&mut f),
            _ => self.fmt(f),
        }
    }
}

impl serde::Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Value::Void(_) => serializer.serialize_unit(),
            Value::I8(me) => serializer.serialize_i8(*me),
            Value::I16(me) => serializer.serialize_i16(*me),
            Value::I32(me) => serializer.serialize_i32(*me),
            Value::I64(me) => serializer.serialize_i64(*me),
            Value::I128(me) => serializer.serialize_i128(*me),
            Value::U8(me) => serializer.serialize_u8(*me),
            Value::U16(me) => serializer.serialize_u16(*me),
            Value::U32(me) => serializer.serialize_u32(*me),
            Value::U64(me) => serializer.serialize_u64(*me),
            Value::U128(me) => serializer.serialize_u128(*me),
            Value::F32(me) => serializer.serialize_f32(*me),
            Value::F64(me) => serializer.serialize_f64(*me),
            Value::Bool(me) => serializer.serialize_bool(*me),
            Value::Byte(me) => serializer.serialize_bytes(&[*me]),
            Value::Char(me) => serializer.serialize_char(*me),
            Value::String(me) => serializer.serialize_str(me),
            Value::Vec(me) => {
                let mut ser = serializer.serialize_seq(Some(me.len()))?;
                for val in me {
                    ser.serialize_element(val)?;
                }
                ser.end()
            }
            Value::Option(me) => {
                if let Some(elmt) = me {
                    serializer.serialize_some(elmt)
                } else {
                    serializer.serialize_none()
                }
            }
            Value::Data(data) => serde::Serialize::serialize(&data, serializer),
        }
    }
}

impl core::fmt::Display for Value {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
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
            Value::Char(v) => write!(
                f,
                "'{}'",
                match v {
                    '\n' => r"\n".to_string(),
                    '\r' => r"\r".to_string(),
                    '\t' => r"\t".to_string(),
                    '\\' => r"\\".to_string(),
                    '\0' => r"\0".to_string(),
                    '\'' => r"\'".to_string(),
                    c if c.is_control() => c.escape_unicode().to_string(),
                    c => c.to_string(),
                }
            ),
            Value::String(v) => {
                if v.chars()
                    .any(|c| c.is_control() && c != '\n' && c != '\t' && c != '\r')
                    || v.contains(['\0'])
                {
                    write!(
                        f,
                        "\"{}\"",
                        v.chars()
                            .map(|c| match c {
                                '\n' => r"\n".to_string(),
                                '\r' => r"\r".to_string(),
                                '\t' => r"\t".to_string(),
                                '\\' => r"\\".to_string(),
                                '\0' => r"\0".to_string(),
                                '\"' => r#"\""#.to_string(),
                                c if c.is_control() => c.escape_unicode().to_string(),
                                c => c.to_string(),
                            })
                            .collect::<String>()
                    )
                } else {
                    let mut start_braces: String = "{".into();
                    let mut end_braces: String = "}".into();
                    while v.contains(&start_braces) || v.contains(&end_braces) {
                        start_braces.push('{');
                        end_braces.push('}');
                    }
                    write!(f, "${start_braces}{v}{end_braces}")
                }
            }
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
                    .map(|val| ToString::to_string(val))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),

            Value::Data(obj) => write!(f, "/* {} */", obj.descriptor().identifier().name()),
        }
    }
}
