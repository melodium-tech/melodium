use super::{Data, DataType, Generic};
use core::fmt::Display;
use melodium_common::descriptor::{
    Collection, DescribedType as CommonDescribedType, Entry as CommonEntry,
    Identifier as CommonIdentifier,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DescribedType {
    Void,

    I8,
    I16,
    I32,
    I64,
    I128,

    U8,
    U16,
    U32,
    U64,
    U128,

    F32,
    F64,

    Bool,
    Byte,

    Char,
    String,

    Vec(Box<DescribedType>),
    Option(Box<DescribedType>),

    Data(Data),

    Generic(Generic),
}

impl DescribedType {
    pub fn to_described_type(&self, collection: &Collection) -> Option<CommonDescribedType> {
        match self {
            DescribedType::Void => Some(CommonDescribedType::Void),

            DescribedType::I8 => Some(CommonDescribedType::I8),
            DescribedType::I16 => Some(CommonDescribedType::I16),
            DescribedType::I32 => Some(CommonDescribedType::I32),
            DescribedType::I64 => Some(CommonDescribedType::I64),
            DescribedType::I128 => Some(CommonDescribedType::I128),

            DescribedType::U8 => Some(CommonDescribedType::U8),
            DescribedType::U16 => Some(CommonDescribedType::U16),
            DescribedType::U32 => Some(CommonDescribedType::U32),
            DescribedType::U64 => Some(CommonDescribedType::U64),
            DescribedType::U128 => Some(CommonDescribedType::U128),

            DescribedType::F32 => Some(CommonDescribedType::F32),
            DescribedType::F64 => Some(CommonDescribedType::F64),

            DescribedType::Bool => Some(CommonDescribedType::Bool),
            DescribedType::Byte => Some(CommonDescribedType::Byte),

            DescribedType::Char => Some(CommonDescribedType::Char),
            DescribedType::String => Some(CommonDescribedType::String),

            DescribedType::Vec(dt) => Some(CommonDescribedType::Vec(Box::new(
                dt.as_ref().to_described_type(collection)?,
            ))),
            DescribedType::Option(dt) => Some(CommonDescribedType::Option(Box::new(
                dt.as_ref().to_described_type(collection)?,
            ))),
            DescribedType::Data(data) => collection
                .get(
                    &TryInto::<CommonIdentifier>::try_into(&data.identifier)
                        .ok()?
                        .into(),
                )
                .map(|entry| {
                    if let CommonEntry::Data(data) = entry {
                        Some(CommonDescribedType::Data(Box::new(Arc::clone(data))))
                    } else {
                        None
                    }
                })
                .flatten(),

            DescribedType::Generic(generic) => {
                Some(CommonDescribedType::Generic(Box::new(generic.into())))
            }
        }
    }
}

impl From<CommonDescribedType> for DescribedType {
    fn from(value: CommonDescribedType) -> Self {
        (&value).into()
    }
}

impl From<&CommonDescribedType> for DescribedType {
    fn from(value: &CommonDescribedType) -> Self {
        match value {
            CommonDescribedType::Void => DescribedType::Void,

            CommonDescribedType::I8 => DescribedType::I8,
            CommonDescribedType::I16 => DescribedType::I16,
            CommonDescribedType::I32 => DescribedType::I32,
            CommonDescribedType::I64 => DescribedType::I64,
            CommonDescribedType::I128 => DescribedType::I128,

            CommonDescribedType::U8 => DescribedType::U8,
            CommonDescribedType::U16 => DescribedType::U16,
            CommonDescribedType::U32 => DescribedType::U32,
            CommonDescribedType::U64 => DescribedType::U64,
            CommonDescribedType::U128 => DescribedType::U128,

            CommonDescribedType::F32 => DescribedType::F32,
            CommonDescribedType::F64 => DescribedType::F64,

            CommonDescribedType::Bool => DescribedType::Bool,
            CommonDescribedType::Byte => DescribedType::Byte,

            CommonDescribedType::Char => DescribedType::Char,
            CommonDescribedType::String => DescribedType::String,
            CommonDescribedType::Vec(dt) => DescribedType::Vec(Box::new(dt.as_ref().into())),
            CommonDescribedType::Option(dt) => DescribedType::Option(Box::new(dt.as_ref().into())),

            CommonDescribedType::Data(data) => DescribedType::Data(data.as_ref().as_ref().into()),
            CommonDescribedType::Generic(generic) => {
                DescribedType::Generic(generic.as_ref().into())
            }
        }
    }
}

impl From<DataType> for DescribedType {
    fn from(value: DataType) -> Self {
        match value {
            DataType::Undetermined => DescribedType::Generic(Generic {
                name: "undetermined".to_string(),
                traits: Vec::new(),
            }),
            DataType::Void => DescribedType::Void,
            DataType::I8 => DescribedType::I8,
            DataType::I16 => DescribedType::I16,
            DataType::I32 => DescribedType::I32,
            DataType::I64 => DescribedType::I64,
            DataType::I128 => DescribedType::I128,
            DataType::U8 => DescribedType::U8,
            DataType::U16 => DescribedType::U16,
            DataType::U32 => DescribedType::U32,
            DataType::U64 => DescribedType::U64,
            DataType::U128 => DescribedType::U128,
            DataType::F32 => DescribedType::F32,
            DataType::F64 => DescribedType::F64,
            DataType::Bool => DescribedType::Bool,
            DataType::Byte => DescribedType::Byte,
            DataType::Char => DescribedType::Char,
            DataType::String => DescribedType::String,
            DataType::Vec(inner) => DescribedType::Vec(Box::new(DescribedType::from(*inner))),
            DataType::Option(inner) => DescribedType::Option(Box::new(DescribedType::from(*inner))),
            DataType::Data(obj) => DescribedType::Data(obj),
        }
    }
}

impl From<&DataType> for DescribedType {
    fn from(value: &DataType) -> Self {
        match value {
            DataType::Undetermined => DescribedType::Generic(Generic {
                name: "undetermined".to_string(),
                traits: Vec::new(),
            }),
            DataType::Void => DescribedType::Void,
            DataType::I8 => DescribedType::I8,
            DataType::I16 => DescribedType::I16,
            DataType::I32 => DescribedType::I32,
            DataType::I64 => DescribedType::I64,
            DataType::I128 => DescribedType::I128,
            DataType::U8 => DescribedType::U8,
            DataType::U16 => DescribedType::U16,
            DataType::U32 => DescribedType::U32,
            DataType::U64 => DescribedType::U64,
            DataType::U128 => DescribedType::U128,
            DataType::F32 => DescribedType::F32,
            DataType::F64 => DescribedType::F64,
            DataType::Bool => DescribedType::Bool,
            DataType::Byte => DescribedType::Byte,
            DataType::Char => DescribedType::Char,
            DataType::String => DescribedType::String,
            DataType::Vec(inner) => {
                DescribedType::Vec(Box::new(DescribedType::from(inner.as_ref())))
            }
            DataType::Option(inner) => {
                DescribedType::Option(Box::new(DescribedType::from(inner.as_ref())))
            }
            DataType::Data(obj) => DescribedType::Data(obj.clone()),
        }
    }
}

impl TryInto<CommonDescribedType> for DescribedType {
    type Error = ();
    fn try_into(self) -> Result<CommonDescribedType, ()> {
        TryInto::try_into(&self)
    }
}

impl TryInto<CommonDescribedType> for &DescribedType {
    type Error = ();
    fn try_into(self) -> Result<CommonDescribedType, ()> {
        match self {
            DescribedType::Void => Ok(CommonDescribedType::Void),

            DescribedType::I8 => Ok(CommonDescribedType::I8),
            DescribedType::I16 => Ok(CommonDescribedType::I16),
            DescribedType::I32 => Ok(CommonDescribedType::I32),
            DescribedType::I64 => Ok(CommonDescribedType::I64),
            DescribedType::I128 => Ok(CommonDescribedType::I128),

            DescribedType::U8 => Ok(CommonDescribedType::U8),
            DescribedType::U16 => Ok(CommonDescribedType::U16),
            DescribedType::U32 => Ok(CommonDescribedType::U32),
            DescribedType::U64 => Ok(CommonDescribedType::U64),
            DescribedType::U128 => Ok(CommonDescribedType::U128),

            DescribedType::F32 => Ok(CommonDescribedType::F32),
            DescribedType::F64 => Ok(CommonDescribedType::F64),

            DescribedType::Bool => Ok(CommonDescribedType::Bool),
            DescribedType::Byte => Ok(CommonDescribedType::Byte),

            DescribedType::Char => Ok(CommonDescribedType::Char),
            DescribedType::String => Ok(CommonDescribedType::String),

            DescribedType::Vec(dt) => {
                Ok(CommonDescribedType::Vec(Box::new(dt.as_ref().try_into()?)))
            }
            DescribedType::Option(dt) => Ok(CommonDescribedType::Option(Box::new(
                dt.as_ref().try_into()?,
            ))),
            DescribedType::Data(_) => Err(()),

            DescribedType::Generic(generic) => {
                Ok(CommonDescribedType::Generic(Box::new(generic.into())))
            }
        }
    }
}

impl Display for DescribedType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DescribedType::Void => write!(f, "void"),
            DescribedType::I8 => write!(f, "i8"),
            DescribedType::I16 => write!(f, "i16"),
            DescribedType::I32 => write!(f, "i32"),
            DescribedType::I64 => write!(f, "i64"),
            DescribedType::I128 => write!(f, "i128"),
            DescribedType::U8 => write!(f, "u8"),
            DescribedType::U16 => write!(f, "u16"),
            DescribedType::U32 => write!(f, "u32"),
            DescribedType::U64 => write!(f, "u64"),
            DescribedType::U128 => write!(f, "u128"),
            DescribedType::F32 => write!(f, "f32"),
            DescribedType::F64 => write!(f, "f64"),
            DescribedType::Bool => write!(f, "bool"),
            DescribedType::Byte => write!(f, "byte"),
            DescribedType::Char => write!(f, "char"),
            DescribedType::String => write!(f, "string"),
            DescribedType::Vec(inner) => write!(f, "Vec<{inner}>"),
            DescribedType::Option(inner) => write!(f, "Option<{inner}>"),
            DescribedType::Data(obj) => write!(f, "{}", obj.identifier.name),
            DescribedType::Generic(gen) => write!(f, "{}", gen.name),
        }
    }
}
