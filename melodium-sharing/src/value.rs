use crate::{DescribedType, Identifier};
use melodium_common::executive::Value as CommonValue;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Value {
    Raw(RawValue),
    Variable(String),
    Context(Identifier, String),
    Function(Identifier, BTreeMap<String, DescribedType>, Vec<Value>),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum RawValue {
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

    Vec(Vec<RawValue>),
    Option(Option<Box<RawValue>>),
}

impl TryFrom<CommonValue> for RawValue {
    type Error = ();
    fn try_from(value: CommonValue) -> Result<Self, ()> {
        match value {
            CommonValue::Void(_) => Ok(RawValue::Void(())),
            CommonValue::I8(n) => Ok(RawValue::I8(n)),
            CommonValue::I16(n) => Ok(RawValue::I16(n)),
            CommonValue::I32(n) => Ok(RawValue::I32(n)),
            CommonValue::I64(n) => Ok(RawValue::I64(n)),
            CommonValue::I128(n) => Ok(RawValue::I128(n)),
            CommonValue::U8(n) => Ok(RawValue::U8(n)),
            CommonValue::U16(n) => Ok(RawValue::U16(n)),
            CommonValue::U32(n) => Ok(RawValue::U32(n)),
            CommonValue::U64(n) => Ok(RawValue::U64(n)),
            CommonValue::U128(n) => Ok(RawValue::U128(n)),
            CommonValue::F32(n) => Ok(RawValue::F32(n)),
            CommonValue::F64(n) => Ok(RawValue::F64(n)),
            CommonValue::Bool(b) => Ok(RawValue::Bool(b)),
            CommonValue::Byte(b) => Ok(RawValue::Byte(b)),
            CommonValue::Char(c) => Ok(RawValue::Char(c)),
            CommonValue::String(s) => Ok(RawValue::String(s)),
            CommonValue::Vec(v) => Ok(RawValue::Vec({
                let mut vec = Vec::new();
                for v in v {
                    vec.push(v.try_into()?);
                }
                vec
            })),
            CommonValue::Option(v) => Ok(RawValue::Option(if let Some(v) = v {
                Some(Box::new((*v).try_into()?))
            } else {
                None
            })),
            CommonValue::Data(_) => Err(()),
        }
    }
}

impl Into<CommonValue> for RawValue {
    fn into(self) -> CommonValue {
        (&self).into()
    }
}

impl Into<CommonValue> for &RawValue {
    fn into(self) -> CommonValue {
        match self {
            RawValue::Void(_) => CommonValue::Void(()),
            RawValue::I8(n) => CommonValue::I8(*n),
            RawValue::I16(n) => CommonValue::I16(*n),
            RawValue::I32(n) => CommonValue::I32(*n),
            RawValue::I64(n) => CommonValue::I64(*n),
            RawValue::I128(n) => CommonValue::I128(*n),
            RawValue::U8(n) => CommonValue::U8(*n),
            RawValue::U16(n) => CommonValue::U16(*n),
            RawValue::U32(n) => CommonValue::U32(*n),
            RawValue::U64(n) => CommonValue::U64(*n),
            RawValue::U128(n) => CommonValue::U128(*n),
            RawValue::F32(n) => CommonValue::F32(*n),
            RawValue::F64(n) => CommonValue::F64(*n),
            RawValue::Bool(b) => CommonValue::Bool(*b),
            RawValue::Byte(b) => CommonValue::Byte(*b),
            RawValue::Char(c) => CommonValue::Char(*c),
            RawValue::String(s) => CommonValue::String(s.clone()),
            RawValue::Vec(v) => CommonValue::Vec(v.iter().map(|v| v.into()).collect()),
            RawValue::Option(v) => {
                CommonValue::Option(v.as_ref().map(|v| Box::new(v.as_ref().into())))
            }
        }
    }
}
