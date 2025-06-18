use super::Data;
use melodium_common::descriptor::{
    Collection, DataType as CommonDataType, Entry as CommonEntry, Identifier as CommonIdentifier,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "webassembly", derive(tsify::Tsify))]
#[cfg_attr(feature = "webassembly", tsify(into_wasm_abi, from_wasm_abi))]
pub enum DataType {
    /// Special variant not aimed to be explicitly used,
    /// it corresponds to the case a value Vec or Option
    /// didn't contain any data, so is not determinable.
    /// It always matches other data type, including itself.
    Undetermined,

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

    Vec(Box<DataType>),
    Option(Box<DataType>),

    Data(Data),
}

impl DataType {
    pub fn to_datatype(&self, collection: &Collection) -> Option<CommonDataType> {
        match self {
            DataType::Undetermined => Some(CommonDataType::Undetermined),

            DataType::Void => Some(CommonDataType::Void),

            DataType::I8 => Some(CommonDataType::I8),
            DataType::I16 => Some(CommonDataType::I16),
            DataType::I32 => Some(CommonDataType::I32),
            DataType::I64 => Some(CommonDataType::I64),
            DataType::I128 => Some(CommonDataType::I128),

            DataType::U8 => Some(CommonDataType::U8),
            DataType::U16 => Some(CommonDataType::U16),
            DataType::U32 => Some(CommonDataType::U32),
            DataType::U64 => Some(CommonDataType::U64),
            DataType::U128 => Some(CommonDataType::U128),

            DataType::F32 => Some(CommonDataType::F32),
            DataType::F64 => Some(CommonDataType::F64),

            DataType::Bool => Some(CommonDataType::Bool),
            DataType::Byte => Some(CommonDataType::Byte),

            DataType::Char => Some(CommonDataType::Char),
            DataType::String => Some(CommonDataType::String),

            DataType::Vec(dt) => Some(CommonDataType::Vec(Box::new(
                dt.as_ref().to_datatype(collection)?,
            ))),
            DataType::Option(dt) => Some(CommonDataType::Option(Box::new(
                dt.as_ref().to_datatype(collection)?,
            ))),
            DataType::Data(data) => collection
                .get(
                    &TryInto::<CommonIdentifier>::try_into(&data.identifier)
                        .ok()?
                        .into(),
                )
                .map(|entry| {
                    if let CommonEntry::Data(data) = entry {
                        Some(CommonDataType::Data(Arc::clone(data)))
                    } else {
                        None
                    }
                })
                .flatten(),
        }
    }
}

impl From<&CommonDataType> for DataType {
    fn from(value: &CommonDataType) -> Self {
        match value {
            CommonDataType::Undetermined => DataType::Undetermined,

            CommonDataType::Void => DataType::Void,

            CommonDataType::I8 => DataType::I8,
            CommonDataType::I16 => DataType::I16,
            CommonDataType::I32 => DataType::I32,
            CommonDataType::I64 => DataType::I64,
            CommonDataType::I128 => DataType::I128,

            CommonDataType::U8 => DataType::U8,
            CommonDataType::U16 => DataType::U16,
            CommonDataType::U32 => DataType::U32,
            CommonDataType::U64 => DataType::U64,
            CommonDataType::U128 => DataType::U128,

            CommonDataType::F32 => DataType::F32,
            CommonDataType::F64 => DataType::F64,

            CommonDataType::Bool => DataType::Bool,
            CommonDataType::Byte => DataType::Byte,

            CommonDataType::Char => DataType::Char,
            CommonDataType::String => DataType::String,

            CommonDataType::Vec(dt) => DataType::Vec(Box::new(dt.as_ref().into())),
            CommonDataType::Option(dt) => DataType::Option(Box::new(dt.as_ref().into())),
            CommonDataType::Data(data) => DataType::Data(data.as_ref().into()),
        }
    }
}

impl TryInto<CommonDataType> for DataType {
    type Error = ();
    fn try_into(self) -> Result<CommonDataType, ()> {
        TryInto::try_into(&self)
    }
}

impl TryInto<CommonDataType> for &DataType {
    type Error = ();
    fn try_into(self) -> Result<CommonDataType, ()> {
        match self {
            DataType::Undetermined => Ok(CommonDataType::Undetermined),

            DataType::Void => Ok(CommonDataType::Void),

            DataType::I8 => Ok(CommonDataType::I8),
            DataType::I16 => Ok(CommonDataType::I16),
            DataType::I32 => Ok(CommonDataType::I32),
            DataType::I64 => Ok(CommonDataType::I64),
            DataType::I128 => Ok(CommonDataType::I128),

            DataType::U8 => Ok(CommonDataType::U8),
            DataType::U16 => Ok(CommonDataType::U16),
            DataType::U32 => Ok(CommonDataType::U32),
            DataType::U64 => Ok(CommonDataType::U64),
            DataType::U128 => Ok(CommonDataType::U128),

            DataType::F32 => Ok(CommonDataType::F32),
            DataType::F64 => Ok(CommonDataType::F64),

            DataType::Bool => Ok(CommonDataType::Bool),
            DataType::Byte => Ok(CommonDataType::Byte),

            DataType::Char => Ok(CommonDataType::Char),
            DataType::String => Ok(CommonDataType::String),

            DataType::Vec(dt) => Ok(CommonDataType::Vec(Box::new(dt.as_ref().try_into()?))),
            DataType::Option(dt) => Ok(CommonDataType::Option(Box::new(dt.as_ref().try_into()?))),
            DataType::Data(_) => Err(()),
        }
    }
}
