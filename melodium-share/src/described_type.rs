use super::{Data, DataType, Generic, SharingError, SharingResult};
use core::fmt::Display;
use melodium_common::descriptor::{
    Collection, DescribedType as CommonDescribedType, Entry as CommonEntry,
    Identifier as CommonIdentifier,
};
use melodium_engine::LogicError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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
    pub fn to_described_type(
        &self,
        collection: &Collection,
        scope: &CommonIdentifier,
    ) -> SharingResult<CommonDescribedType> {
        match self {
            DescribedType::Void => SharingResult::new_success(CommonDescribedType::Void),

            DescribedType::I8 => SharingResult::new_success(CommonDescribedType::I8),
            DescribedType::I16 => SharingResult::new_success(CommonDescribedType::I16),
            DescribedType::I32 => SharingResult::new_success(CommonDescribedType::I32),
            DescribedType::I64 => SharingResult::new_success(CommonDescribedType::I64),
            DescribedType::I128 => SharingResult::new_success(CommonDescribedType::I128),

            DescribedType::U8 => SharingResult::new_success(CommonDescribedType::U8),
            DescribedType::U16 => SharingResult::new_success(CommonDescribedType::U16),
            DescribedType::U32 => SharingResult::new_success(CommonDescribedType::U32),
            DescribedType::U64 => SharingResult::new_success(CommonDescribedType::U64),
            DescribedType::U128 => SharingResult::new_success(CommonDescribedType::U128),

            DescribedType::F32 => SharingResult::new_success(CommonDescribedType::F32),
            DescribedType::F64 => SharingResult::new_success(CommonDescribedType::F64),

            DescribedType::Bool => SharingResult::new_success(CommonDescribedType::Bool),
            DescribedType::Byte => SharingResult::new_success(CommonDescribedType::Byte),

            DescribedType::Char => SharingResult::new_success(CommonDescribedType::Char),
            DescribedType::String => SharingResult::new_success(CommonDescribedType::String),

            DescribedType::Vec(dt) => {
                SharingResult::new_success(CommonDescribedType::Vec(Box::new({
                    let result = dt.as_ref().to_described_type(collection, scope);
                    if let Some(subtype) = result.success() {
                        subtype.clone()
                    } else {
                        return result;
                    }
                })))
            }
            DescribedType::Option(dt) => {
                SharingResult::new_success(CommonDescribedType::Option(Box::new({
                    let result = dt.as_ref().to_described_type(collection, scope);
                    if let Some(subtype) = result.success() {
                        subtype.clone()
                    } else {
                        return result;
                    }
                })))
            }
            DescribedType::Data(data) => {
                let identifier: CommonIdentifier =
                    if let Ok(identifier) = (&data.identifier).try_into() {
                        identifier
                    } else {
                        return SharingResult::new_failure(SharingError::invalid_identifier(
                            5,
                            data.identifier.clone(),
                        ));
                    };
                if let Some(CommonEntry::Data(data)) = collection.get(&(&identifier).into()) {
                    SharingResult::new_success(CommonDescribedType::Data(Box::new(Arc::clone(
                        data,
                    ))))
                } else {
                    return SharingResult::new_failure(
                        LogicError::unexisting_data(231, scope.clone(), identifier.into(), None)
                            .into(),
                    );
                }
            }

            DescribedType::Generic(generic) => {
                SharingResult::new_success(CommonDescribedType::Generic(Box::new(generic.into())))
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
