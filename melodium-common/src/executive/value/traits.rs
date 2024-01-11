use super::Value;
use crate::executive::DataTrait;
use core::convert::TryInto;

impl DataTrait for Value {
    fn to_i8(&self) -> Value {
        match self {
            Value::I8(val) => Value::I8(*val),
            other => panic!("ToI8 not supported for {}", other.datatype()),
        }
    }

    fn to_i16(&self) -> Value {
        match self {
            Value::I8(val) => Value::I16(*val as i16),
            Value::I16(val) => Value::I16(*val),
            Value::U8(val) => Value::I16(*val as i16),
            other => panic!("ToI16 not supported for {}", other.datatype()),
        }
    }

    fn to_i32(&self) -> Value {
        match self {
            Value::I8(val) => Value::I32(*val as i32),
            Value::I16(val) => Value::I32(*val as i32),
            Value::I32(val) => Value::I32(*val),
            Value::U8(val) => Value::I32(*val as i32),
            Value::U16(val) => Value::I32(*val as i32),
            other => panic!("ToI32 not supported for {}", other.datatype()),
        }
    }

    fn to_i64(&self) -> Value {
        match self {
            Value::I8(val) => Value::I64(*val as i64),
            Value::I16(val) => Value::I64(*val as i64),
            Value::I32(val) => Value::I64(*val as i64),
            Value::I64(val) => Value::I64(*val),
            Value::U8(val) => Value::I64(*val as i64),
            Value::U16(val) => Value::I64(*val as i64),
            Value::U32(val) => Value::I64(*val as i64),
            other => panic!("ToI64 not supported for {}", other.datatype()),
        }
    }

    fn to_i128(&self) -> Value {
        match self {
            Value::I8(val) => Value::I128(*val as i128),
            Value::I16(val) => Value::I128(*val as i128),
            Value::I32(val) => Value::I128(*val as i128),
            Value::I64(val) => Value::I128(*val as i128),
            Value::U8(val) => Value::I128(*val as i128),
            Value::U16(val) => Value::I128(*val as i128),
            Value::U32(val) => Value::I128(*val as i128),
            Value::U64(val) => Value::I128(*val as i128),
            Value::I128(val) => Value::I128(*val),
            other => panic!("ToI128 not supported for {}", other.datatype()),
        }
    }

    fn to_u8(&self) -> Value {
        match self {
            Value::U8(val) => Value::U8(*val),
            Value::Byte(val) => Value::U8(*val),
            other => panic!("ToU8 not supported for {}", other.datatype()),
        }
    }

    fn to_u16(&self) -> Value {
        match self {
            Value::U8(val) => Value::U16(*val as u16),
            Value::U16(val) => Value::U16(*val),
            other => panic!("ToU16 not supported for {}", other.datatype()),
        }
    }

    fn to_u32(&self) -> Value {
        match self {
            Value::U8(val) => Value::U32(*val as u32),
            Value::U16(val) => Value::U32(*val as u32),
            Value::U32(val) => Value::U32(*val),
            other => panic!("ToU32 not supported for {}", other.datatype()),
        }
    }

    fn to_u64(&self) -> Value {
        match self {
            Value::U8(val) => Value::U64(*val as u64),
            Value::U16(val) => Value::U64(*val as u64),
            Value::U32(val) => Value::U64(*val as u64),
            Value::U64(val) => Value::U64(*val),
            other => panic!("ToU64 not supported for {}", other.datatype()),
        }
    }

    fn to_u128(&self) -> Value {
        match self {
            Value::U8(val) => Value::U128(*val as u128),
            Value::U16(val) => Value::U128(*val as u128),
            Value::U32(val) => Value::U128(*val as u128),
            Value::U64(val) => Value::U128(*val as u128),
            Value::U128(val) => Value::U128(*val),
            other => panic!("ToU128 not supported for {}", other.datatype()),
        }
    }

    fn to_f32(&self) -> Value {
        match self {
            Value::I8(val) => Value::F32(*val as f32),
            Value::I16(val) => Value::F32(*val as f32),
            Value::I32(val) => Value::F32(*val as f32),
            Value::I64(val) => Value::F32(*val as f32),
            Value::I128(val) => Value::F32(*val as f32),
            Value::U8(val) => Value::F32(*val as f32),
            Value::U16(val) => Value::F32(*val as f32),
            Value::U32(val) => Value::F32(*val as f32),
            Value::U64(val) => Value::F32(*val as f32),
            Value::U128(val) => Value::F32(*val as f32),
            Value::F32(val) => Value::F32(*val),
            Value::F64(val) => Value::F32(*val as f32),
            other => panic!("ToF32 not supported for {}", other.datatype()),
        }
    }

    fn to_f64(&self) -> Value {
        match self {
            Value::I8(val) => Value::F64(*val as f64),
            Value::I16(val) => Value::F64(*val as f64),
            Value::I32(val) => Value::F64(*val as f64),
            Value::I64(val) => Value::F64(*val as f64),
            Value::I128(val) => Value::F64(*val as f64),
            Value::U8(val) => Value::F64(*val as f64),
            Value::U16(val) => Value::F64(*val as f64),
            Value::U32(val) => Value::F64(*val as f64),
            Value::U64(val) => Value::F64(*val as f64),
            Value::U128(val) => Value::F64(*val as f64),
            Value::F32(val) => Value::F64(*val as f64),
            Value::F64(val) => Value::F64(*val),
            other => panic!("ToF64 not supported for {}", other.datatype()),
        }
    }

    fn to_bool(&self) -> Value {
        match self {
            Value::I8(val) => Value::Bool(*val != 0),
            Value::I16(val) => Value::Bool(*val != 0),
            Value::I32(val) => Value::Bool(*val != 0),
            Value::I64(val) => Value::Bool(*val != 0),
            Value::I128(val) => Value::Bool(*val != 0),
            Value::U8(val) => Value::Bool(*val != 0),
            Value::U16(val) => Value::Bool(*val != 0),
            Value::U32(val) => Value::Bool(*val != 0),
            Value::U64(val) => Value::Bool(*val != 0),
            Value::U128(val) => Value::Bool(*val != 0),
            Value::Bool(val) => Value::Bool(*val),
            Value::Byte(val) => Value::Bool(*val != 0),
            other => panic!("ToBool not supported for {}", other.datatype()),
        }
    }

    fn to_byte(&self) -> Value {
        match self {
            Value::U8(val) => Value::Byte(*val),
            Value::Bool(val) => Value::Byte(*val as u8),
            Value::Byte(val) => Value::Byte(*val),
            other => panic!("ToByte not supported for {}", other.datatype()),
        }
    }

    fn to_char(&self) -> Value {
        match self {
            Value::Char(val) => Value::Char(*val),
            other => panic!("ToChar not supported for {}", other.datatype()),
        }
    }

    fn to_string(&self) -> Value {
        match self {
            Value::Char(val) => Value::String(val.to_string()),
            Value::String(val) => Value::String(val.clone()),
            other => panic!("ToString not supported for {}", other.datatype()),
        }
    }

    fn try_to_i8(&self) -> Value {
        match self {
            Value::I8(val) => Value::Option(Some(Box::new(Value::I8(*val)))),
            Value::I16(val) => Value::Option(
                TryInto::<i8>::try_into(*val)
                    .map(|val| Box::new(Value::I8(val)))
                    .ok(),
            ),
            Value::I32(val) => Value::Option(
                TryInto::<i8>::try_into(*val)
                    .map(|val| Box::new(Value::I8(val)))
                    .ok(),
            ),
            Value::I64(val) => Value::Option(
                TryInto::<i8>::try_into(*val)
                    .map(|val| Box::new(Value::I8(val)))
                    .ok(),
            ),
            Value::I128(val) => Value::Option(
                TryInto::<i8>::try_into(*val)
                    .map(|val| Box::new(Value::I8(val)))
                    .ok(),
            ),
            Value::U8(val) => Value::Option(
                TryInto::<i8>::try_into(*val)
                    .map(|val| Box::new(Value::I8(val)))
                    .ok(),
            ),
            Value::U16(val) => Value::Option(
                TryInto::<i8>::try_into(*val)
                    .map(|val| Box::new(Value::I8(val)))
                    .ok(),
            ),
            Value::U32(val) => Value::Option(
                TryInto::<i8>::try_into(*val)
                    .map(|val| Box::new(Value::I8(val)))
                    .ok(),
            ),
            Value::U64(val) => Value::Option(
                TryInto::<i8>::try_into(*val)
                    .map(|val| Box::new(Value::I8(val)))
                    .ok(),
            ),
            Value::U128(val) => Value::Option(
                TryInto::<i8>::try_into(*val)
                    .map(|val| Box::new(Value::I8(val)))
                    .ok(),
            ),
            Value::F32(val) => Value::Option(
                if val.is_finite() && *val <= i8::MAX as f32 && *val >= i8::MIN as f32 {
                    Some(Box::new(Value::I8(*val as i8)))
                } else {
                    None
                },
            ),
            Value::F64(val) => Value::Option(
                if val.is_finite() && *val <= i8::MAX as f64 && *val >= i8::MIN as f64 {
                    Some(Box::new(Value::I8(*val as i8)))
                } else {
                    None
                },
            ),
            other => panic!("TryToI8 not supported for {}", other.datatype()),
        }
    }

    fn try_to_i16(&self) -> Value {
        match self {
            Value::I8(val) => Value::Option(Some(Box::new(Value::I16(*val as i16)))),
            Value::I16(val) => Value::Option(Some(Box::new(Value::I16(*val)))),
            Value::I32(val) => Value::Option(
                TryInto::<i16>::try_into(*val)
                    .map(|val| Box::new(Value::I16(val)))
                    .ok(),
            ),
            Value::I64(val) => Value::Option(
                TryInto::<i16>::try_into(*val)
                    .map(|val| Box::new(Value::I16(val)))
                    .ok(),
            ),
            Value::I128(val) => Value::Option(
                TryInto::<i16>::try_into(*val)
                    .map(|val| Box::new(Value::I16(val)))
                    .ok(),
            ),
            Value::U8(val) => Value::Option(
                TryInto::<i16>::try_into(*val)
                    .map(|val| Box::new(Value::I16(val)))
                    .ok(),
            ),
            Value::U16(val) => Value::Option(
                TryInto::<i16>::try_into(*val)
                    .map(|val| Box::new(Value::I16(val)))
                    .ok(),
            ),
            Value::U32(val) => Value::Option(
                TryInto::<i16>::try_into(*val)
                    .map(|val| Box::new(Value::I16(val)))
                    .ok(),
            ),
            Value::U64(val) => Value::Option(
                TryInto::<i16>::try_into(*val)
                    .map(|val| Box::new(Value::I16(val)))
                    .ok(),
            ),
            Value::U128(val) => Value::Option(
                TryInto::<i16>::try_into(*val)
                    .map(|val| Box::new(Value::I16(val)))
                    .ok(),
            ),
            Value::F32(val) => Value::Option(
                if val.is_finite() && *val <= i16::MAX as f32 && *val >= i16::MIN as f32 {
                    Some(Box::new(Value::I16(*val as i16)))
                } else {
                    None
                },
            ),
            Value::F64(val) => Value::Option(
                if val.is_finite() && *val <= i16::MAX as f64 && *val >= i16::MIN as f64 {
                    Some(Box::new(Value::I16(*val as i16)))
                } else {
                    None
                },
            ),
            other => panic!("TryToI16 not supported for {}", other.datatype()),
        }
    }

    fn try_to_i32(&self) -> Value {
        match self {
            Value::I8(val) => Value::Option(Some(Box::new(Value::I32(*val as i32)))),
            Value::I16(val) => Value::Option(Some(Box::new(Value::I32(*val as i32)))),
            Value::I32(val) => Value::Option(Some(Box::new(Value::I32(*val)))),
            Value::I64(val) => Value::Option(
                TryInto::<i32>::try_into(*val)
                    .map(|val| Box::new(Value::I32(val)))
                    .ok(),
            ),
            Value::I128(val) => Value::Option(
                TryInto::<i32>::try_into(*val)
                    .map(|val| Box::new(Value::I32(val)))
                    .ok(),
            ),
            Value::U8(val) => Value::Option(
                TryInto::<i32>::try_into(*val)
                    .map(|val| Box::new(Value::I32(val)))
                    .ok(),
            ),
            Value::U16(val) => Value::Option(
                TryInto::<i32>::try_into(*val)
                    .map(|val| Box::new(Value::I32(val)))
                    .ok(),
            ),
            Value::U32(val) => Value::Option(
                TryInto::<i32>::try_into(*val)
                    .map(|val| Box::new(Value::I32(val)))
                    .ok(),
            ),
            Value::U64(val) => Value::Option(
                TryInto::<i32>::try_into(*val)
                    .map(|val| Box::new(Value::I32(val)))
                    .ok(),
            ),
            Value::U128(val) => Value::Option(
                TryInto::<i32>::try_into(*val)
                    .map(|val| Box::new(Value::I32(val)))
                    .ok(),
            ),
            Value::F32(val) => Value::Option(
                if val.is_finite() && *val <= i32::MAX as f32 && *val >= i32::MIN as f32 {
                    Some(Box::new(Value::I32(*val as i32)))
                } else {
                    None
                },
            ),
            Value::F64(val) => Value::Option(
                if val.is_finite() && *val <= i32::MAX as f64 && *val >= i32::MIN as f64 {
                    Some(Box::new(Value::I32(*val as i32)))
                } else {
                    None
                },
            ),
            other => panic!("TryToI32 not supported for {}", other.datatype()),
        }
    }

    fn try_to_i64(&self) -> Value {
        match self {
            Value::I8(val) => Value::Option(Some(Box::new(Value::I64(*val as i64)))),
            Value::I16(val) => Value::Option(Some(Box::new(Value::I64(*val as i64)))),
            Value::I32(val) => Value::Option(Some(Box::new(Value::I64(*val as i64)))),
            Value::I64(val) => Value::Option(Some(Box::new(Value::I64(*val)))),
            Value::I128(val) => Value::Option(
                TryInto::<i64>::try_into(*val)
                    .map(|val| Box::new(Value::I64(val)))
                    .ok(),
            ),
            Value::U8(val) => Value::Option(
                TryInto::<i64>::try_into(*val)
                    .map(|val| Box::new(Value::I64(val)))
                    .ok(),
            ),
            Value::U16(val) => Value::Option(
                TryInto::<i64>::try_into(*val)
                    .map(|val| Box::new(Value::I64(val)))
                    .ok(),
            ),
            Value::U32(val) => Value::Option(
                TryInto::<i64>::try_into(*val)
                    .map(|val| Box::new(Value::I64(val)))
                    .ok(),
            ),
            Value::U64(val) => Value::Option(
                TryInto::<i64>::try_into(*val)
                    .map(|val| Box::new(Value::I64(val)))
                    .ok(),
            ),
            Value::U128(val) => Value::Option(
                TryInto::<i64>::try_into(*val)
                    .map(|val| Box::new(Value::I64(val)))
                    .ok(),
            ),
            Value::F32(val) => Value::Option(
                if val.is_finite() && *val <= i64::MAX as f32 && *val >= i64::MIN as f32 {
                    Some(Box::new(Value::I64(*val as i64)))
                } else {
                    None
                },
            ),
            Value::F64(val) => Value::Option(
                if val.is_finite() && *val <= i64::MAX as f64 && *val >= i64::MIN as f64 {
                    Some(Box::new(Value::I64(*val as i64)))
                } else {
                    None
                },
            ),
            other => panic!("TryToI64 not supported for {}", other.datatype()),
        }
    }

    fn try_to_i128(&self) -> Value {
        match self {
            Value::I8(val) => Value::Option(Some(Box::new(Value::I128(*val as i128)))),
            Value::I16(val) => Value::Option(Some(Box::new(Value::I128(*val as i128)))),
            Value::I32(val) => Value::Option(Some(Box::new(Value::I128(*val as i128)))),
            Value::I64(val) => Value::Option(Some(Box::new(Value::I128(*val as i128)))),
            Value::I128(val) => Value::Option(Some(Box::new(Value::I128(*val)))),
            Value::U8(val) => Value::Option(
                TryInto::<i128>::try_into(*val)
                    .map(|val| Box::new(Value::I128(val)))
                    .ok(),
            ),
            Value::U16(val) => Value::Option(
                TryInto::<i128>::try_into(*val)
                    .map(|val| Box::new(Value::I128(val)))
                    .ok(),
            ),
            Value::U32(val) => Value::Option(
                TryInto::<i128>::try_into(*val)
                    .map(|val| Box::new(Value::I128(val)))
                    .ok(),
            ),
            Value::U64(val) => Value::Option(
                TryInto::<i128>::try_into(*val)
                    .map(|val| Box::new(Value::I128(val)))
                    .ok(),
            ),
            Value::U128(val) => Value::Option(
                TryInto::<i128>::try_into(*val)
                    .map(|val| Box::new(Value::I128(val)))
                    .ok(),
            ),
            Value::F32(val) => Value::Option(
                if val.is_finite() && *val <= i128::MAX as f32 && *val >= i128::MIN as f32 {
                    Some(Box::new(Value::I128(*val as i128)))
                } else {
                    None
                },
            ),
            Value::F64(val) => Value::Option(
                if val.is_finite() && *val <= i128::MAX as f64 && *val >= i128::MIN as f64 {
                    Some(Box::new(Value::I128(*val as i128)))
                } else {
                    None
                },
            ),
            other => panic!("TryToI128 not supported for {}", other.datatype()),
        }
    }

    fn try_to_u8(&self) -> Value {
        match self {
            Value::I8(val) => Value::Option(
                TryInto::<u8>::try_into(*val)
                    .map(|val| Box::new(Value::U8(val)))
                    .ok(),
            ),
            Value::I16(val) => Value::Option(
                TryInto::<u8>::try_into(*val)
                    .map(|val| Box::new(Value::U8(val)))
                    .ok(),
            ),
            Value::I32(val) => Value::Option(
                TryInto::<u8>::try_into(*val)
                    .map(|val| Box::new(Value::U8(val)))
                    .ok(),
            ),
            Value::I64(val) => Value::Option(
                TryInto::<u8>::try_into(*val)
                    .map(|val| Box::new(Value::U8(val)))
                    .ok(),
            ),
            Value::I128(val) => Value::Option(
                TryInto::<u8>::try_into(*val)
                    .map(|val| Box::new(Value::U8(val)))
                    .ok(),
            ),
            Value::U8(val) => Value::Option(Some(Box::new(Value::U8(*val)))),
            Value::U16(val) => Value::Option(
                TryInto::<u8>::try_into(*val)
                    .map(|val| Box::new(Value::U8(val)))
                    .ok(),
            ),
            Value::U32(val) => Value::Option(
                TryInto::<u8>::try_into(*val)
                    .map(|val| Box::new(Value::U8(val)))
                    .ok(),
            ),
            Value::U64(val) => Value::Option(
                TryInto::<u8>::try_into(*val)
                    .map(|val| Box::new(Value::U8(val)))
                    .ok(),
            ),
            Value::U128(val) => Value::Option(
                TryInto::<u8>::try_into(*val)
                    .map(|val| Box::new(Value::U8(val)))
                    .ok(),
            ),
            Value::F32(val) => Value::Option(
                if val.is_finite() && *val <= u8::MAX as f32 && *val >= u8::MIN as f32 {
                    Some(Box::new(Value::U8(*val as u8)))
                } else {
                    None
                },
            ),
            Value::F64(val) => Value::Option(
                if val.is_finite() && *val <= u8::MAX as f64 && *val >= u8::MIN as f64 {
                    Some(Box::new(Value::U8(*val as u8)))
                } else {
                    None
                },
            ),
            other => panic!("TryToU8 not supported for {}", other.datatype()),
        }
    }

    fn try_to_u16(&self) -> Value {
        match self {
            Value::I8(val) => Value::Option(
                TryInto::<u16>::try_into(*val)
                    .map(|val| Box::new(Value::U16(val)))
                    .ok(),
            ),
            Value::I16(val) => Value::Option(
                TryInto::<u16>::try_into(*val)
                    .map(|val| Box::new(Value::U16(val)))
                    .ok(),
            ),
            Value::I32(val) => Value::Option(
                TryInto::<u16>::try_into(*val)
                    .map(|val| Box::new(Value::U16(val)))
                    .ok(),
            ),
            Value::I64(val) => Value::Option(
                TryInto::<u16>::try_into(*val)
                    .map(|val| Box::new(Value::U16(val)))
                    .ok(),
            ),
            Value::I128(val) => Value::Option(
                TryInto::<u16>::try_into(*val)
                    .map(|val| Box::new(Value::U16(val)))
                    .ok(),
            ),
            Value::U8(val) => Value::Option(Some(Box::new(Value::U16(*val as u16)))),
            Value::U16(val) => Value::Option(Some(Box::new(Value::U16(*val)))),
            Value::U32(val) => Value::Option(
                TryInto::<u16>::try_into(*val)
                    .map(|val| Box::new(Value::U16(val)))
                    .ok(),
            ),
            Value::U64(val) => Value::Option(
                TryInto::<u16>::try_into(*val)
                    .map(|val| Box::new(Value::U16(val)))
                    .ok(),
            ),
            Value::U128(val) => Value::Option(
                TryInto::<u16>::try_into(*val)
                    .map(|val| Box::new(Value::U16(val)))
                    .ok(),
            ),
            Value::F32(val) => Value::Option(
                if val.is_finite() && *val <= u16::MAX as f32 && *val >= u16::MIN as f32 {
                    Some(Box::new(Value::U16(*val as u16)))
                } else {
                    None
                },
            ),
            Value::F64(val) => Value::Option(
                if val.is_finite() && *val <= u16::MAX as f64 && *val >= u16::MIN as f64 {
                    Some(Box::new(Value::U16(*val as u16)))
                } else {
                    None
                },
            ),
            other => panic!("TryToU16 not supported for {}", other.datatype()),
        }
    }

    fn try_to_u32(&self) -> Value {
        match self {
            Value::I8(val) => Value::Option(
                TryInto::<u32>::try_into(*val)
                    .map(|val| Box::new(Value::U32(val)))
                    .ok(),
            ),
            Value::I16(val) => Value::Option(
                TryInto::<u32>::try_into(*val)
                    .map(|val| Box::new(Value::U32(val)))
                    .ok(),
            ),
            Value::I32(val) => Value::Option(
                TryInto::<u32>::try_into(*val)
                    .map(|val| Box::new(Value::U32(val)))
                    .ok(),
            ),
            Value::I64(val) => Value::Option(
                TryInto::<u32>::try_into(*val)
                    .map(|val| Box::new(Value::U32(val)))
                    .ok(),
            ),
            Value::I128(val) => Value::Option(
                TryInto::<u32>::try_into(*val)
                    .map(|val| Box::new(Value::U32(val)))
                    .ok(),
            ),
            Value::U8(val) => Value::Option(Some(Box::new(Value::U32(*val as u32)))),
            Value::U16(val) => Value::Option(Some(Box::new(Value::U32(*val as u32)))),
            Value::U32(val) => Value::Option(Some(Box::new(Value::U32(*val)))),
            Value::U64(val) => Value::Option(
                TryInto::<u32>::try_into(*val)
                    .map(|val| Box::new(Value::U32(val)))
                    .ok(),
            ),
            Value::U128(val) => Value::Option(
                TryInto::<u32>::try_into(*val)
                    .map(|val| Box::new(Value::U32(val)))
                    .ok(),
            ),
            Value::F32(val) => Value::Option(
                if val.is_finite() && *val <= u32::MAX as f32 && *val >= u32::MIN as f32 {
                    Some(Box::new(Value::U32(*val as u32)))
                } else {
                    None
                },
            ),
            Value::F64(val) => Value::Option(
                if val.is_finite() && *val <= u32::MAX as f64 && *val >= u32::MIN as f64 {
                    Some(Box::new(Value::U32(*val as u32)))
                } else {
                    None
                },
            ),
            other => panic!("TryToU32 not supported for {}", other.datatype()),
        }
    }

    fn try_to_u64(&self) -> Value {
        match self {
            Value::I8(val) => Value::Option(
                TryInto::<u64>::try_into(*val)
                    .map(|val| Box::new(Value::U64(val)))
                    .ok(),
            ),
            Value::I16(val) => Value::Option(
                TryInto::<u64>::try_into(*val)
                    .map(|val| Box::new(Value::U64(val)))
                    .ok(),
            ),
            Value::I32(val) => Value::Option(
                TryInto::<u64>::try_into(*val)
                    .map(|val| Box::new(Value::U64(val)))
                    .ok(),
            ),
            Value::I64(val) => Value::Option(
                TryInto::<u64>::try_into(*val)
                    .map(|val| Box::new(Value::U64(val)))
                    .ok(),
            ),
            Value::I128(val) => Value::Option(
                TryInto::<u64>::try_into(*val)
                    .map(|val| Box::new(Value::U64(val)))
                    .ok(),
            ),
            Value::U8(val) => Value::Option(Some(Box::new(Value::U64(*val as u64)))),
            Value::U16(val) => Value::Option(Some(Box::new(Value::U64(*val as u64)))),
            Value::U32(val) => Value::Option(Some(Box::new(Value::U64(*val as u64)))),
            Value::U64(val) => Value::Option(Some(Box::new(Value::U64(*val)))),
            Value::U128(val) => Value::Option(
                TryInto::<u64>::try_into(*val)
                    .map(|val| Box::new(Value::U64(val)))
                    .ok(),
            ),
            Value::F32(val) => Value::Option(
                if val.is_finite() && *val <= u64::MAX as f32 && *val >= u64::MIN as f32 {
                    Some(Box::new(Value::U64(*val as u64)))
                } else {
                    None
                },
            ),
            Value::F64(val) => Value::Option(
                if val.is_finite() && *val <= u64::MAX as f64 && *val >= u64::MIN as f64 {
                    Some(Box::new(Value::U64(*val as u64)))
                } else {
                    None
                },
            ),
            other => panic!("TryToU64 not supported for {}", other.datatype()),
        }
    }

    fn try_to_u128(&self) -> Value {
        match self {
            Value::I8(val) => Value::Option(
                TryInto::<u128>::try_into(*val)
                    .map(|val| Box::new(Value::U128(val)))
                    .ok(),
            ),
            Value::I16(val) => Value::Option(
                TryInto::<u128>::try_into(*val)
                    .map(|val| Box::new(Value::U128(val)))
                    .ok(),
            ),
            Value::I32(val) => Value::Option(
                TryInto::<u128>::try_into(*val)
                    .map(|val| Box::new(Value::U128(val)))
                    .ok(),
            ),
            Value::I64(val) => Value::Option(
                TryInto::<u128>::try_into(*val)
                    .map(|val| Box::new(Value::U128(val)))
                    .ok(),
            ),
            Value::I128(val) => Value::Option(
                TryInto::<u128>::try_into(*val)
                    .map(|val| Box::new(Value::U128(val)))
                    .ok(),
            ),
            Value::U8(val) => Value::Option(Some(Box::new(Value::U128(*val as u128)))),
            Value::U16(val) => Value::Option(Some(Box::new(Value::U128(*val as u128)))),
            Value::U32(val) => Value::Option(Some(Box::new(Value::U128(*val as u128)))),
            Value::U64(val) => Value::Option(Some(Box::new(Value::U128(*val as u128)))),
            Value::U128(val) => Value::Option(Some(Box::new(Value::U128(*val)))),
            Value::F32(val) => Value::Option(
                if val.is_finite() && *val <= u128::MAX as f32 && *val >= u128::MIN as f32 {
                    Some(Box::new(Value::U128(*val as u128)))
                } else {
                    None
                },
            ),
            Value::F64(val) => Value::Option(
                if val.is_finite() && *val <= u128::MAX as f64 && *val >= u128::MIN as f64 {
                    Some(Box::new(Value::U128(*val as u128)))
                } else {
                    None
                },
            ),
            other => panic!("TryToU128 not supported for {}", other.datatype()),
        }
    }

    fn try_to_f32(&self) -> Value {
        match self {
            Value::I8(val) => Value::Option(Some(Box::new(Value::F32(*val as f32)))),
            Value::I16(val) => Value::Option(Some(Box::new(Value::F32(*val as f32)))),
            Value::I32(val) => Value::Option(Some(Box::new(Value::F32(*val as f32)))),
            Value::I64(val) => Value::Option(Some(Box::new(Value::F32(*val as f32)))),
            Value::I128(val) => Value::Option(Some(Box::new(Value::F32(*val as f32)))),
            Value::U8(val) => Value::Option(Some(Box::new(Value::F32(*val as f32)))),
            Value::U16(val) => Value::Option(Some(Box::new(Value::F32(*val as f32)))),
            Value::U32(val) => Value::Option(Some(Box::new(Value::F32(*val as f32)))),
            Value::U64(val) => Value::Option(Some(Box::new(Value::F32(*val as f32)))),
            Value::U128(val) => Value::Option(Some(Box::new(Value::F32(*val as f32)))),
            Value::F32(val) => Value::Option(Some(Box::new(Value::F32(*val)))),
            Value::F64(val) => Value::Option(Some(Box::new(Value::F32(*val as f32)))),
            other => panic!("TryToF32 not supported for {}", other.datatype()),
        }
    }

    fn try_to_f64(&self) -> Value {
        match self {
            Value::I8(val) => Value::Option(Some(Box::new(Value::F64(*val as f64)))),
            Value::I16(val) => Value::Option(Some(Box::new(Value::F64(*val as f64)))),
            Value::I32(val) => Value::Option(Some(Box::new(Value::F64(*val as f64)))),
            Value::I64(val) => Value::Option(Some(Box::new(Value::F64(*val as f64)))),
            Value::I128(val) => Value::Option(Some(Box::new(Value::F64(*val as f64)))),
            Value::U8(val) => Value::Option(Some(Box::new(Value::F64(*val as f64)))),
            Value::U16(val) => Value::Option(Some(Box::new(Value::F64(*val as f64)))),
            Value::U32(val) => Value::Option(Some(Box::new(Value::F64(*val as f64)))),
            Value::U64(val) => Value::Option(Some(Box::new(Value::F64(*val as f64)))),
            Value::U128(val) => Value::Option(Some(Box::new(Value::F64(*val as f64)))),
            Value::F32(val) => Value::Option(Some(Box::new(Value::F64(*val as f64)))),
            Value::F64(val) => Value::Option(Some(Box::new(Value::F64(*val)))),
            other => panic!("TryToF32 not supported for {}", other.datatype()),
        }
    }

    fn try_to_bool(&self) -> Value {
        match self {
            Value::I8(val) => Value::Option(Some(Box::new(Value::Bool(*val != 0)))),
            Value::I16(val) => Value::Option(Some(Box::new(Value::Bool(*val != 0)))),
            Value::I32(val) => Value::Option(Some(Box::new(Value::Bool(*val != 0)))),
            Value::I64(val) => Value::Option(Some(Box::new(Value::Bool(*val != 0)))),
            Value::I128(val) => Value::Option(Some(Box::new(Value::Bool(*val != 0)))),
            Value::U8(val) => Value::Option(Some(Box::new(Value::Bool(*val != 0)))),
            Value::U16(val) => Value::Option(Some(Box::new(Value::Bool(*val != 0)))),
            Value::U32(val) => Value::Option(Some(Box::new(Value::Bool(*val != 0)))),
            Value::U64(val) => Value::Option(Some(Box::new(Value::Bool(*val != 0)))),
            Value::U128(val) => Value::Option(Some(Box::new(Value::Bool(*val != 0)))),
            Value::Bool(val) => Value::Option(Some(Box::new(Value::Bool(*val)))),
            Value::Byte(val) => Value::Option(Some(Box::new(Value::Bool(*val != 0)))),
            other => panic!("TryToBool not supported for {}", other.datatype()),
        }
    }

    fn try_to_byte(&self) -> Value {
        match self {
            Value::U8(val) => Value::Option(Some(Box::new(Value::Byte(*val)))),
            Value::Bool(val) => Value::Option(Some(Box::new(Value::Byte(*val as u8)))),
            Value::Byte(val) => Value::Option(Some(Box::new(Value::Byte(*val)))),
            other => panic!("TryToByte not supported for {}", other.datatype()),
        }
    }

    fn try_to_char(&self) -> Value {
        match self {
            Value::Char(val) => Value::Option(Some(Box::new(Value::Char(*val)))),
            other => panic!("TryToChar not supported for {}", other.datatype()),
        }
    }

    fn try_to_string(&self) -> Value {
        match self {
            Value::Char(val) => Value::Option(Some(Box::new(Value::String(val.to_string())))),
            Value::String(val) => Value::Option(Some(Box::new(Value::String(val.clone())))),
            other => panic!("TryToString not supported for {}", other.datatype()),
        }
    }

    fn saturating_to_i8(&self) -> Value {
        match self {
            Value::I8(val) => Value::I8(*val),
            Value::I16(val) => Value::I8(match *val {
                val if val < i8::MIN as i16 => i8::MIN,
                val if val > i8::MAX as i16 => i8::MAX,
                val => val as i8,
            }),
            Value::I32(val) => Value::I8(match *val {
                val if val < i8::MIN as i32 => i8::MIN,
                val if val > i8::MAX as i32 => i8::MAX,
                val => val as i8,
            }),
            Value::I64(val) => Value::I8(match *val {
                val if val < i8::MIN as i64 => i8::MIN,
                val if val > i8::MAX as i64 => i8::MAX,
                val => val as i8,
            }),
            Value::I128(val) => Value::I8(match *val {
                val if val < i8::MIN as i128 => i8::MIN,
                val if val > i8::MAX as i128 => i8::MAX,
                val => val as i8,
            }),
            Value::U8(val) => Value::I8(match *val {
                val if val > i8::MAX as u8 => i8::MAX,
                val => val as i8,
            }),
            Value::U16(val) => Value::I8(match *val {
                val if val > i8::MAX as u16 => i8::MAX,
                val => val as i8,
            }),
            Value::U32(val) => Value::I8(match *val {
                val if val > i8::MAX as u32 => i8::MAX,
                val => val as i8,
            }),
            Value::U64(val) => Value::I8(match *val {
                val if val > i8::MAX as u64 => i8::MAX,
                val => val as i8,
            }),
            Value::U128(val) => Value::I8(match *val {
                val if val > i8::MAX as u128 => i8::MAX,
                val => val as i8,
            }),
            Value::F32(val) => Value::I8(if val.is_nan() {
                0
            } else if *val < i8::MIN as f32 {
                i8::MIN
            } else if *val > i8::MAX as f32 {
                i8::MAX
            } else {
                *val as i8
            }),
            Value::F64(val) => Value::I8(if val.is_nan() {
                0
            } else if *val < i8::MIN as f64 {
                i8::MIN
            } else if *val > i8::MAX as f64 {
                i8::MAX
            } else {
                *val as i8
            }),
            other => panic!("SaturatingToI8 not supported for {}", other.datatype()),
        }
    }

    fn saturating_to_i16(&self) -> Value {
        match self {
            Value::I8(val) => Value::I16(*val as i16),
            Value::I16(val) => Value::I16(*val),
            Value::I32(val) => Value::I16(match *val {
                val if val < i16::MIN as i32 => i16::MIN,
                val if val > i16::MAX as i32 => i16::MAX,
                val => val as i16,
            }),
            Value::I64(val) => Value::I16(match *val {
                val if val < i16::MIN as i64 => i16::MIN,
                val if val > i16::MAX as i64 => i16::MAX,
                val => val as i16,
            }),
            Value::I128(val) => Value::I16(match *val {
                val if val < i16::MIN as i128 => i16::MIN,
                val if val > i16::MAX as i128 => i16::MAX,
                val => val as i16,
            }),
            Value::U8(val) => Value::I16(*val as i16),
            Value::U16(val) => Value::I16(match *val {
                val if val > i16::MAX as u16 => i16::MAX,
                val => val as i16,
            }),
            Value::U32(val) => Value::I16(match *val {
                val if val > i16::MAX as u32 => i16::MAX,
                val => val as i16,
            }),
            Value::U64(val) => Value::I16(match *val {
                val if val > i16::MAX as u64 => i16::MAX,
                val => val as i16,
            }),
            Value::U128(val) => Value::I16(match *val {
                val if val > i16::MAX as u128 => i16::MAX,
                val => val as i16,
            }),
            Value::F32(val) => Value::I16(if val.is_nan() {
                0
            } else if *val < i16::MIN as f32 {
                i16::MIN
            } else if *val > i16::MAX as f32 {
                i16::MAX
            } else {
                *val as i16
            }),
            Value::F64(val) => Value::I16(if val.is_nan() {
                0
            } else if *val < i16::MIN as f64 {
                i16::MIN
            } else if *val > i16::MAX as f64 {
                i16::MAX
            } else {
                *val as i16
            }),
            other => panic!("SaturatingToI16 not supported for {}", other.datatype()),
        }
    }

    fn saturating_to_i32(&self) -> Value {
        match self {
            Value::I8(val) => Value::I32(*val as i32),
            Value::I16(val) => Value::I32(*val as i32),
            Value::I32(val) => Value::I32(*val),
            Value::I64(val) => Value::I32(match *val {
                val if val < i32::MIN as i64 => i32::MIN,
                val if val > i32::MAX as i64 => i32::MAX,
                val => val as i32,
            }),
            Value::I128(val) => Value::I32(match *val {
                val if val < i32::MIN as i128 => i32::MIN,
                val if val > i32::MAX as i128 => i32::MAX,
                val => val as i32,
            }),
            Value::U8(val) => Value::I32(*val as i32),
            Value::U16(val) => Value::I32(*val as i32),
            Value::U32(val) => Value::I32(match *val {
                val if val > i32::MAX as u32 => i32::MAX,
                val => val as i32,
            }),
            Value::U64(val) => Value::I32(match *val {
                val if val > i32::MAX as u64 => i32::MAX,
                val => val as i32,
            }),
            Value::U128(val) => Value::I32(match *val {
                val if val > i32::MAX as u128 => i32::MAX,
                val => val as i32,
            }),
            Value::F32(val) => Value::I32(if val.is_nan() {
                0
            } else if *val < i32::MIN as f32 {
                i32::MIN
            } else if *val > i32::MAX as f32 {
                i32::MAX
            } else {
                *val as i32
            }),
            Value::F64(val) => Value::I32(if val.is_nan() {
                0
            } else if *val < i32::MIN as f64 {
                i32::MIN
            } else if *val > i32::MAX as f64 {
                i32::MAX
            } else {
                *val as i32
            }),
            other => panic!("SaturatingToI32 not supported for {}", other.datatype()),
        }
    }

    fn saturating_to_i64(&self) -> Value {
        match self {
            Value::I8(val) => Value::I64(*val as i64),
            Value::I16(val) => Value::I64(*val as i64),
            Value::I32(val) => Value::I64(*val as i64),
            Value::I64(val) => Value::I64(*val),
            Value::I128(val) => Value::I64(match *val {
                val if val < i64::MIN as i128 => i64::MIN,
                val if val > i64::MAX as i128 => i64::MAX,
                val => val as i64,
            }),
            Value::U8(val) => Value::I64(*val as i64),
            Value::U16(val) => Value::I64(*val as i64),
            Value::U32(val) => Value::I64(*val as i64),
            Value::U64(val) => Value::I64(match *val {
                val if val > i64::MAX as u64 => i64::MAX,
                val => val as i64,
            }),
            Value::U128(val) => Value::I64(match *val {
                val if val > i64::MAX as u128 => i64::MAX,
                val => val as i64,
            }),
            Value::F32(val) => Value::I64(if val.is_nan() {
                0
            } else if *val < i64::MIN as f32 {
                i64::MIN
            } else if *val > i64::MAX as f32 {
                i64::MAX
            } else {
                *val as i64
            }),
            Value::F64(val) => Value::I64(if val.is_nan() {
                0
            } else if *val < i64::MIN as f64 {
                i64::MIN
            } else if *val > i64::MAX as f64 {
                i64::MAX
            } else {
                *val as i64
            }),
            other => panic!("SaturatingToI64 not supported for {}", other.datatype()),
        }
    }

    fn saturating_to_i128(&self) -> Value {
        match self {
            Value::I8(val) => Value::I128(*val as i128),
            Value::I16(val) => Value::I128(*val as i128),
            Value::I32(val) => Value::I128(*val as i128),
            Value::I64(val) => Value::I128(*val as i128),
            Value::I128(val) => Value::I128(*val),
            Value::U8(val) => Value::I128(*val as i128),
            Value::U16(val) => Value::I128(*val as i128),
            Value::U32(val) => Value::I128(*val as i128),
            Value::U64(val) => Value::I128(*val as i128),
            Value::U128(val) => Value::I128(match *val {
                val if val > i128::MAX as u128 => i128::MAX,
                val => val as i128,
            }),
            Value::F32(val) => Value::I128(if val.is_nan() {
                0
            } else if *val < i128::MIN as f32 {
                i128::MIN
            } else if *val > i128::MAX as f32 {
                i128::MAX
            } else {
                *val as i128
            }),
            Value::F64(val) => Value::I128(if val.is_nan() {
                0
            } else if *val < i128::MIN as f64 {
                i128::MIN
            } else if *val > i128::MAX as f64 {
                i128::MAX
            } else {
                *val as i128
            }),
            other => panic!("SaturatingToI128 not supported for {}", other.datatype()),
        }
    }

    fn saturating_to_u8(&self) -> Value {
        match self {
            Value::I8(val) => Value::U8(match *val {
                val if val < u8::MIN as i8 => u8::MIN,
                val => val as u8,
            }),
            Value::I16(val) => Value::U8(match *val {
                val if val < u8::MIN as i16 => u8::MIN,
                val if val > u8::MAX as i16 => u8::MAX,
                val => val as u8,
            }),
            Value::I32(val) => Value::U8(match *val {
                val if val < u8::MIN as i32 => u8::MIN,
                val if val > u8::MAX as i32 => u8::MAX,
                val => val as u8,
            }),
            Value::I64(val) => Value::U8(match *val {
                val if val < u8::MIN as i64 => u8::MIN,
                val if val > u8::MAX as i64 => u8::MAX,
                val => val as u8,
            }),
            Value::I128(val) => Value::U8(match *val {
                val if val < u8::MIN as i128 => u8::MIN,
                val if val > u8::MAX as i128 => u8::MAX,
                val => val as u8,
            }),
            Value::U8(val) => Value::U8(*val),
            Value::U16(val) => Value::U8(match *val {
                val if val > u8::MAX as u16 => u8::MAX,
                val => val as u8,
            }),
            Value::U32(val) => Value::U8(match *val {
                val if val > u8::MAX as u32 => u8::MAX,
                val => val as u8,
            }),
            Value::U64(val) => Value::U8(match *val {
                val if val > u8::MAX as u64 => u8::MAX,
                val => val as u8,
            }),
            Value::U128(val) => Value::U8(match *val {
                val if val > u8::MAX as u128 => u8::MAX,
                val => val as u8,
            }),
            Value::F32(val) => Value::U8(if val.is_nan() {
                0
            } else if *val < u8::MIN as f32 {
                u8::MIN
            } else if *val > u8::MAX as f32 {
                u8::MAX
            } else {
                *val as u8
            }),
            Value::F64(val) => Value::U8(if val.is_nan() {
                0
            } else if *val < u8::MIN as f64 {
                u8::MIN
            } else if *val > u8::MAX as f64 {
                u8::MAX
            } else {
                *val as u8
            }),
            other => panic!("SaturatingToU8 not supported for {}", other.datatype()),
        }
    }

    fn saturating_to_u16(&self) -> Value {
        match self {
            Value::I8(val) => Value::U16(match *val {
                val if val < u16::MIN as i8 => u16::MIN,
                val => val as u16,
            }),
            Value::I16(val) => Value::U16(match *val {
                val if val < u16::MIN as i16 => u16::MIN,
                val => val as u16,
            }),
            Value::I32(val) => Value::U16(match *val {
                val if val < u16::MIN as i32 => u16::MIN,
                val if val > u16::MAX as i32 => u16::MAX,
                val => val as u16,
            }),
            Value::I64(val) => Value::U16(match *val {
                val if val < u16::MIN as i64 => u16::MIN,
                val if val > u16::MAX as i64 => u16::MAX,
                val => val as u16,
            }),
            Value::I128(val) => Value::U16(match *val {
                val if val < u16::MIN as i128 => u16::MIN,
                val if val > u16::MAX as i128 => u16::MAX,
                val => val as u16,
            }),
            Value::U8(val) => Value::U16(*val as u16),
            Value::U16(val) => Value::U16(*val),
            Value::U32(val) => Value::U16(match *val {
                val if val > u16::MAX as u32 => u16::MAX,
                val => val as u16,
            }),
            Value::U64(val) => Value::U16(match *val {
                val if val > u16::MAX as u64 => u16::MAX,
                val => val as u16,
            }),
            Value::U128(val) => Value::U16(match *val {
                val if val > u16::MAX as u128 => u16::MAX,
                val => val as u16,
            }),
            Value::F32(val) => Value::U16(if val.is_nan() {
                0
            } else if *val < u16::MIN as f32 {
                u16::MIN
            } else if *val > u16::MAX as f32 {
                u16::MAX
            } else {
                *val as u16
            }),
            Value::F64(val) => Value::U16(if val.is_nan() {
                0
            } else if *val < u16::MIN as f64 {
                u16::MIN
            } else if *val > u16::MAX as f64 {
                u16::MAX
            } else {
                *val as u16
            }),
            other => panic!("SaturatingToU16 not supported for {}", other.datatype()),
        }
    }

    fn saturating_to_u32(&self) -> Value {
        match self {
            Value::I8(val) => Value::U32(match *val {
                val if val < u32::MIN as i8 => u32::MIN,
                val => val as u32,
            }),
            Value::I16(val) => Value::U32(match *val {
                val if val < u32::MIN as i16 => u32::MIN,
                val => val as u32,
            }),
            Value::I32(val) => Value::U32(match *val {
                val if val < u32::MIN as i32 => u32::MIN,
                val => val as u32,
            }),
            Value::I64(val) => Value::U32(match *val {
                val if val < u32::MIN as i64 => u32::MIN,
                val if val > u32::MAX as i64 => u32::MAX,
                val => val as u32,
            }),
            Value::I128(val) => Value::U32(match *val {
                val if val < u32::MIN as i128 => u32::MIN,
                val if val > u32::MAX as i128 => u32::MAX,
                val => val as u32,
            }),
            Value::U8(val) => Value::U32(*val as u32),
            Value::U16(val) => Value::U32(*val as u32),
            Value::U32(val) => Value::U32(*val),
            Value::U64(val) => Value::U32(match *val {
                val if val > u32::MAX as u64 => u32::MAX,
                val => val as u32,
            }),
            Value::U128(val) => Value::U32(match *val {
                val if val > u32::MAX as u128 => u32::MAX,
                val => val as u32,
            }),
            Value::F32(val) => Value::U32(if val.is_nan() {
                0
            } else if *val < u32::MIN as f32 {
                u32::MIN
            } else if *val > u32::MAX as f32 {
                u32::MAX
            } else {
                *val as u32
            }),
            Value::F64(val) => Value::U32(if val.is_nan() {
                0
            } else if *val < u32::MIN as f64 {
                u32::MIN
            } else if *val > u32::MAX as f64 {
                u32::MAX
            } else {
                *val as u32
            }),
            other => panic!("SaturatingToU32 not supported for {}", other.datatype()),
        }
    }

    fn saturating_to_u64(&self) -> Value {
        match self {
            Value::I8(val) => Value::U64(match *val {
                val if val < u64::MIN as i8 => u64::MIN,
                val => val as u64,
            }),
            Value::I16(val) => Value::U64(match *val {
                val if val < u64::MIN as i16 => u64::MIN,
                val => val as u64,
            }),
            Value::I32(val) => Value::U64(match *val {
                val if val < u64::MIN as i32 => u64::MIN,
                val => val as u64,
            }),
            Value::I64(val) => Value::U64(match *val {
                val if val < u64::MIN as i64 => u64::MIN,
                val => val as u64,
            }),
            Value::I128(val) => Value::U64(match *val {
                val if val < u64::MIN as i128 => u64::MIN,
                val if val > u64::MAX as i128 => u64::MAX,
                val => val as u64,
            }),
            Value::U8(val) => Value::U64(*val as u64),
            Value::U16(val) => Value::U64(*val as u64),
            Value::U32(val) => Value::U64(*val as u64),
            Value::U64(val) => Value::U64(*val),
            Value::U128(val) => Value::U64(match *val {
                val if val > u64::MAX as u128 => u64::MAX,
                val => val as u64,
            }),
            Value::F32(val) => Value::U64(if val.is_nan() {
                0
            } else if *val < u64::MIN as f32 {
                u64::MIN
            } else if *val > u64::MAX as f32 {
                u64::MAX
            } else {
                *val as u64
            }),
            Value::F64(val) => Value::U64(if val.is_nan() {
                0
            } else if *val < u64::MIN as f64 {
                u64::MIN
            } else if *val > u64::MAX as f64 {
                u64::MAX
            } else {
                *val as u64
            }),
            other => panic!("SaturatingToU64 not supported for {}", other.datatype()),
        }
    }

    fn saturating_to_u128(&self) -> Value {
        match self {
            Value::I8(val) => Value::U128(match *val {
                val if val < u128::MIN as i8 => u128::MIN,
                val => val as u128,
            }),
            Value::I16(val) => Value::U128(match *val {
                val if val < u128::MIN as i16 => u128::MIN,
                val => val as u128,
            }),
            Value::I32(val) => Value::U128(match *val {
                val if val < u128::MIN as i32 => u128::MIN,
                val => val as u128,
            }),
            Value::I64(val) => Value::U128(match *val {
                val if val < u128::MIN as i64 => u128::MIN,
                val => val as u128,
            }),
            Value::I128(val) => Value::U128(match *val {
                val if val < u128::MIN as i128 => u128::MIN,
                val => val as u128,
            }),
            Value::U8(val) => Value::U128(*val as u128),
            Value::U16(val) => Value::U128(*val as u128),
            Value::U32(val) => Value::U128(*val as u128),
            Value::U64(val) => Value::U128(*val as u128),
            Value::U128(val) => Value::U128(*val),
            Value::F32(val) => Value::U128(if val.is_nan() {
                0
            } else if *val < u128::MIN as f32 {
                u128::MIN
            } else if *val > u128::MAX as f32 {
                u128::MAX
            } else {
                *val as u128
            }),
            Value::F64(val) => Value::U128(if val.is_nan() {
                0
            } else if *val < u128::MIN as f64 {
                u128::MIN
            } else if *val > u128::MAX as f64 {
                u128::MAX
            } else {
                *val as u128
            }),
            other => panic!("SaturatingToU128 not supported for {}", other.datatype()),
        }
    }

    fn saturating_to_f32(&self) -> Value {
        todo!()
    }

    fn saturating_to_f64(&self) -> Value {
        todo!()
    }

    fn signed_abs(&self) -> Value {
        todo!()
    }

    fn signed_abs_sub(&self, other: &Value) -> Value {
        todo!()
    }

    fn signed_signum(&self) -> Value {
        todo!()
    }

    fn signed_is_positive(&self, other: &Value) -> Value {
        todo!()
    }

    fn signed_is_negative(&self, other: &Value) -> Value {
        todo!()
    }

    fn float_is_nan(&self) -> Value {
        todo!()
    }

    fn float_is_infinite(&self) -> Value {
        todo!()
    }

    fn float_is_finite(&self) -> Value {
        todo!()
    }

    fn float_is_normal(&self) -> Value {
        todo!()
    }

    fn float_is_subnormal(&self) -> Value {
        todo!()
    }

    fn float_floor(&self) -> Value {
        todo!()
    }

    fn float_ceil(&self) -> Value {
        todo!()
    }

    fn float_round(&self) -> Value {
        todo!()
    }

    fn float_trunc(&self) -> Value {
        todo!()
    }

    fn float_fract(&self) -> Value {
        todo!()
    }

    fn float_recip(&self) -> Value {
        todo!()
    }

    fn float_pow(&self, n: &Value) -> Value {
        todo!()
    }

    fn float_sqrt(&self) -> Value {
        todo!()
    }

    fn float_exp(&self) -> Value {
        todo!()
    }

    fn float_exp2(&self) -> Value {
        todo!()
    }

    fn float_ln(&self) -> Value {
        todo!()
    }

    fn float_log(&self, base: &Value) -> Value {
        todo!()
    }

    fn float_log2(&self) -> Value {
        todo!()
    }

    fn float_log10(&self) -> Value {
        todo!()
    }

    fn float_cbrt(&self) -> Value {
        todo!()
    }

    fn float_hypot(&self) -> Value {
        todo!()
    }

    fn float_sin(&self) -> Value {
        todo!()
    }

    fn float_cos(&self) -> Value {
        todo!()
    }

    fn float_tan(&self) -> Value {
        todo!()
    }

    fn float_asin(&self) -> Value {
        todo!()
    }

    fn float_acos(&self) -> Value {
        todo!()
    }

    fn float_atan(&self) -> Value {
        todo!()
    }

    fn float_atan2(&self) -> Value {
        todo!()
    }

    fn float_sinh(&self) -> Value {
        todo!()
    }

    fn float_cosh(&self) -> Value {
        todo!()
    }

    fn float_tanh(&self) -> Value {
        todo!()
    }

    fn float_asinh(&self) -> Value {
        todo!()
    }

    fn float_acosh(&self) -> Value {
        todo!()
    }

    fn float_atanh(&self) -> Value {
        todo!()
    }

    fn float_to_degrees(&self) -> Value {
        todo!()
    }

    fn float_to_radians(&self) -> Value {
        todo!()
    }

    fn partial_equality_eq(&self, other: &Value) -> Value {
        todo!()
    }

    fn partial_equality_ne(&self, other: &Value) -> Value {
        todo!()
    }

    fn partial_order_lt(&self, other: &Value) -> Value {
        todo!()
    }

    fn partial_order_le(&self, other: &Value) -> Value {
        todo!()
    }

    fn partial_order_gt(&self, other: &Value) -> Value {
        todo!()
    }

    fn partial_order_ge(&self, other: &Value) -> Value {
        todo!()
    }

    fn order_max(&self, other: &Value) -> Value {
        todo!()
    }

    fn order_min(&self, other: &Value) -> Value {
        todo!()
    }

    fn order_clamp(&self, min: &Value, max: &Value) -> Value {
        todo!()
    }

    fn add(&self, other: &Value) -> Value {
        todo!()
    }

    fn checked_add(&self, other: &Value) -> Value {
        todo!()
    }

    fn saturating_add(&self, other: &Value) -> Value {
        todo!()
    }

    fn wrapping_add(&self, other: &Value) -> Value {
        todo!()
    }

    fn sub(&self, other: &Value) -> Value {
        todo!()
    }

    fn checked_sub(&self, other: &Value) -> Value {
        todo!()
    }

    fn saturating_sub(&self, other: &Value) -> Value {
        todo!()
    }

    fn wrapping_sub(&self, other: &Value) -> Value {
        todo!()
    }

    fn mul(&self, other: &Value) -> Value {
        todo!()
    }

    fn checked_mul(&self, other: &Value) -> Value {
        todo!()
    }

    fn saturating_mul(&self, other: &Value) -> Value {
        todo!()
    }

    fn wrapping_mul(&self, other: &Value) -> Value {
        todo!()
    }

    fn div(&self, other: &Value) -> Value {
        todo!()
    }

    fn checked_div(&self, other: &Value) -> Value {
        todo!()
    }

    fn rem(&self, other: &Value) -> Value {
        todo!()
    }

    fn checked_rem(&self, other: &Value) -> Value {
        todo!()
    }

    fn neg(&self) -> Value {
        todo!()
    }

    fn checked_neg(&self) -> Value {
        todo!()
    }

    fn wrapping_neg(&self) -> Value {
        todo!()
    }

    fn pow(&self, exp: &Value) -> Value {
        todo!()
    }

    fn checked_pow(&self, exp: &Value) -> Value {
        todo!()
    }

    fn euclid_div(&self, other: &Value) -> Value {
        todo!()
    }

    fn euclid_rem(&self, other: &Value) -> Value {
        todo!()
    }

    fn checked_euclid_div(&self, other: &Value) -> Value {
        todo!()
    }

    fn checked_euclid_rem(&self, other: &Value) -> Value {
        todo!()
    }
}

impl core::ops::Add for Value {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::I8(me), Value::I8(other)) => Value::I8(me + other),
            (Value::I16(me), Value::I16(other)) => Value::I16(me + other),
            (Value::I32(me), Value::I32(other)) => Value::I32(me + other),
            (Value::I64(me), Value::I64(other)) => Value::I64(me + other),
            (Value::I128(me), Value::I128(other)) => Value::I128(me + other),

            (Value::U8(me), Value::U8(other)) => Value::U8(me + other),
            (Value::U16(me), Value::U16(other)) => Value::U16(me + other),
            (Value::U32(me), Value::U32(other)) => Value::U32(me + other),
            (Value::U64(me), Value::U64(other)) => Value::U64(me + other),
            (Value::U128(me), Value::U128(other)) => Value::U128(me + other),

            (Value::F32(me), Value::F32(other)) => Value::F32(me + other),
            (Value::F64(me), Value::F64(other)) => Value::F64(me + other),
            _ => panic!("Illegal `add` call, this is an internal bug to report."),
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
                    .map(|val| ToString::to_string(val))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }
}
