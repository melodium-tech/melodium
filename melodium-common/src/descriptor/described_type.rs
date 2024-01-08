use super::DataType;
use core::fmt::{Display, Formatter, Result};
use std::collections::HashMap;

#[derive(Clone, PartialEq, Hash, Debug)]
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

    Generic(String),
}

impl DescribedType {
    pub fn contains_generic(&self) -> bool {
        match self {
            DescribedType::Option(me) => me.contains_generic(),
            DescribedType::Vec(me) => me.contains_generic(),
            DescribedType::Generic(_) => true,
            _ => false,
        }
    }

    pub fn is_datatype(&self, dt: &DataType, generics: &HashMap<String, DescribedType>) -> bool {
        if let Some(me) = self.to_datatype(generics) {
            &me == dt
        } else {
            false
        }
    }

    pub fn to_datatype(&self, generics: &HashMap<String, DescribedType>) -> Option<DataType> {
        match self {
            DescribedType::Void => Some(DataType::Void),

            DescribedType::I8 => Some(DataType::I8),
            DescribedType::I16 => Some(DataType::I16),
            DescribedType::I32 => Some(DataType::I32),
            DescribedType::I64 => Some(DataType::I64),
            DescribedType::I128 => Some(DataType::I128),

            DescribedType::U8 => Some(DataType::U8),
            DescribedType::U16 => Some(DataType::U16),
            DescribedType::U32 => Some(DataType::U32),
            DescribedType::U64 => Some(DataType::U64),
            DescribedType::U128 => Some(DataType::U128),

            DescribedType::F32 => Some(DataType::F32),
            DescribedType::F64 => Some(DataType::F64),

            DescribedType::Bool => Some(DataType::Bool),
            DescribedType::Byte => Some(DataType::Byte),

            DescribedType::Char => Some(DataType::Char),
            DescribedType::String => Some(DataType::String),

            DescribedType::Option(me) => me
                .to_datatype(generics)
                .map(|dt| DataType::Option(Box::new(dt))),
            DescribedType::Vec(me) => me
                .to_datatype(generics)
                .map(|dt| DataType::Vec(Box::new(dt))),
            DescribedType::Generic(generic) => generics
                .get(generic)
                .and_then(|me| me.to_datatype(generics)),
        }
    }

    pub fn is_compatible(
        &self,
        generics: &HashMap<String, DescribedType>,
        other: &DescribedType,
        generics_other: &HashMap<String, DescribedType>,
    ) -> bool {
        match (
            self.to_datatype(generics),
            other.to_datatype(generics_other),
        ) {
            (Some(me), Some(other)) => me == other,
            (None, None) => match (self, other) {
                (DescribedType::Generic(me), DescribedType::Generic(other)) => {
                    generics.get(me) == generics_other.get(other)
                }
                _ => false,
            },
            _ => false,
        }
    }

    pub fn as_defined(&self, generics: &HashMap<String, DescribedType>) -> Option<DescribedType> {
        match self {
            DescribedType::Generic(generic) => generics.get(generic).cloned(),
            me => Some(me.clone()),
        }
    }
}

impl From<&DataType> for DescribedType {
    fn from(value: &DataType) -> Self {
        match value {
            DataType::Undetermined => panic!("Undetermined data type"),
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
            DataType::Vec(inner) => DescribedType::Vec(Box::new(DescribedType::from(&**inner))),
            DataType::Option(inner) => {
                DescribedType::Option(Box::new(DescribedType::from(&**inner)))
            }
        }
    }
}

impl From<DataType> for DescribedType {
    fn from(value: DataType) -> Self {
        match value {
            DataType::Undetermined => DescribedType::Generic("undertermined".to_string()),
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
        }
    }
}

impl Display for DescribedType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
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
            DescribedType::Generic(gen) => write!(f, "{}", gen),
        }
    }
}
