use crate::{DescribedType, Identifier};
use ciborium::{cbor, Value as DataValue};
use melodium_common::{descriptor::Collection, executive::Value as CommonValue};
use melodium_engine::design::Value as DesignedValue;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Value {
    Raw(RawValue),
    Array(Vec<Value>),
    Variable(String),
    Context(Identifier, String),
    Function(Identifier, BTreeMap<String, DescribedType>, Vec<Value>),
}

impl From<&DesignedValue> for Value {
    fn from(value: &DesignedValue) -> Self {
        match value {
            DesignedValue::Raw(val) => Value::Raw(val.into()),
            DesignedValue::Array(arr) => Value::Array(arr.iter().map(|v| v.into()).collect()),
            DesignedValue::Variable(var) => Value::Variable(var.clone()),
            DesignedValue::Context(context, name) => {
                Value::Context(context.identifier().into(), name.clone())
            }
            DesignedValue::Function(function, generics, params) => Value::Function(
                function.identifier().into(),
                generics
                    .iter()
                    .map(|(name, dt)| (name.clone(), dt.into()))
                    .collect(),
                params.iter().map(|p| p.into()).collect(),
            ),
        }
    }
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

    Data(Identifier, Option<DataValue>),
}

impl RawValue {
    pub fn to_value(&self, collection: &Collection) -> Option<CommonValue> {
        match self {
            RawValue::Data(identifier, value) => {
                // TODO Manage data deserialization #81
                None
            }
            other => other.try_into().ok(),
        }
    }
}

impl From<CommonValue> for RawValue {
    fn from(value: CommonValue) -> Self {
        match value {
            CommonValue::Void(_) => RawValue::Void(()),
            CommonValue::I8(n) => RawValue::I8(n),
            CommonValue::I16(n) => RawValue::I16(n),
            CommonValue::I32(n) => RawValue::I32(n),
            CommonValue::I64(n) => RawValue::I64(n),
            CommonValue::I128(n) => RawValue::I128(n),
            CommonValue::U8(n) => RawValue::U8(n),
            CommonValue::U16(n) => RawValue::U16(n),
            CommonValue::U32(n) => RawValue::U32(n),
            CommonValue::U64(n) => RawValue::U64(n),
            CommonValue::U128(n) => RawValue::U128(n),
            CommonValue::F32(n) => RawValue::F32(n),
            CommonValue::F64(n) => RawValue::F64(n),
            CommonValue::Bool(b) => RawValue::Bool(b),
            CommonValue::Byte(b) => RawValue::Byte(b),
            CommonValue::Char(c) => RawValue::Char(c),
            CommonValue::String(s) => RawValue::String(s),
            CommonValue::Vec(v) => RawValue::Vec(v.into_iter().map(|v| v.into()).collect()),
            CommonValue::Option(v) => RawValue::Option(v.map(|v| Box::new((*v).into()))),
            CommonValue::Data(d) => {
                RawValue::Data(d.descriptor().identifier().into(), cbor!(d).ok())
            }
        }
    }
}

impl From<&CommonValue> for RawValue {
    fn from(value: &CommonValue) -> Self {
        match value {
            CommonValue::Void(_) => RawValue::Void(()),
            CommonValue::I8(n) => RawValue::I8(*n),
            CommonValue::I16(n) => RawValue::I16(*n),
            CommonValue::I32(n) => RawValue::I32(*n),
            CommonValue::I64(n) => RawValue::I64(*n),
            CommonValue::I128(n) => RawValue::I128(*n),
            CommonValue::U8(n) => RawValue::U8(*n),
            CommonValue::U16(n) => RawValue::U16(*n),
            CommonValue::U32(n) => RawValue::U32(*n),
            CommonValue::U64(n) => RawValue::U64(*n),
            CommonValue::U128(n) => RawValue::U128(*n),
            CommonValue::F32(n) => RawValue::F32(*n),
            CommonValue::F64(n) => RawValue::F64(*n),
            CommonValue::Bool(b) => RawValue::Bool(*b),
            CommonValue::Byte(b) => RawValue::Byte(*b),
            CommonValue::Char(c) => RawValue::Char(*c),
            CommonValue::String(s) => RawValue::String(s.clone()),
            CommonValue::Vec(v) => RawValue::Vec(v.into_iter().map(|v| v.into()).collect()),
            CommonValue::Option(v) => {
                RawValue::Option(v.as_ref().map(|v| Box::new(v.as_ref().into())))
            }
            CommonValue::Data(d) => {
                RawValue::Data(d.descriptor().identifier().into(), cbor!(d).ok())
            }
        }
    }
}

impl TryInto<CommonValue> for RawValue {
    type Error = ();

    fn try_into(self) -> Result<CommonValue, Self::Error> {
        (&self).try_into()
    }
}

impl TryInto<CommonValue> for &RawValue {
    type Error = ();

    fn try_into(self) -> Result<CommonValue, Self::Error> {
        match self {
            RawValue::Void(_) => Ok(CommonValue::Void(())),
            RawValue::I8(n) => Ok(CommonValue::I8(*n)),
            RawValue::I16(n) => Ok(CommonValue::I16(*n)),
            RawValue::I32(n) => Ok(CommonValue::I32(*n)),
            RawValue::I64(n) => Ok(CommonValue::I64(*n)),
            RawValue::I128(n) => Ok(CommonValue::I128(*n)),
            RawValue::U8(n) => Ok(CommonValue::U8(*n)),
            RawValue::U16(n) => Ok(CommonValue::U16(*n)),
            RawValue::U32(n) => Ok(CommonValue::U32(*n)),
            RawValue::U64(n) => Ok(CommonValue::U64(*n)),
            RawValue::U128(n) => Ok(CommonValue::U128(*n)),
            RawValue::F32(n) => Ok(CommonValue::F32(*n)),
            RawValue::F64(n) => Ok(CommonValue::F64(*n)),
            RawValue::Bool(b) => Ok(CommonValue::Bool(*b)),
            RawValue::Byte(b) => Ok(CommonValue::Byte(*b)),
            RawValue::Char(c) => Ok(CommonValue::Char(*c)),
            RawValue::String(s) => Ok(CommonValue::String(s.clone())),
            RawValue::Vec(v) => Ok({
                let mut vec = Vec::with_capacity(v.len());
                for val in v {
                    vec.push(val.try_into()?);
                }
                CommonValue::Vec(vec)
            }),
            RawValue::Option(v) => {
                if let Some(val) = v {
                    Ok(CommonValue::Option(Some(Box::new(
                        val.as_ref().try_into()?,
                    ))))
                } else {
                    Ok(CommonValue::Option(None))
                }
            }
            RawValue::Data(_, _) => Err(()),
        }
    }
}
