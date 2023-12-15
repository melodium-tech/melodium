use core::fmt::{Display, Formatter, Result};
use std::collections::HashMap;

#[derive(Clone, PartialEq, Hash, Debug)]
pub enum DescribedType {
    Concrete(DataType),
    Generic(String),
}

impl DescribedType {
    pub fn is_datatype(&self, dt: &DataType, generics: &HashMap<String, DescribedType>) -> bool {
        match self {
            DescribedType::Concrete(me) => me == dt,
            DescribedType::Generic(generic) => generics
                .get(generic)
                .map(|me| match me {
                    DescribedType::Concrete(me) => me == dt,
                    DescribedType::Generic(_) => false,
                })
                .unwrap_or(false),
        }
    }

    pub fn is_compatible(
        &self,
        other: &DescribedType,
        generics: &HashMap<String, DescribedType>,
    ) -> bool {
        match (self, other) {
            (DescribedType::Concrete(me), DescribedType::Concrete(other)) => me == other,
            (DescribedType::Generic(generic), DescribedType::Concrete(other)) => generics
                .get(generic)
                .map(|me| match me {
                    DescribedType::Concrete(me) => me == other,
                    DescribedType::Generic(_) => false,
                })
                .unwrap_or(false),
            (DescribedType::Concrete(_), DescribedType::Generic(_)) => false,
            (DescribedType::Generic(generic), DescribedType::Generic(other)) => generics
                .get(generic)
                .map(|me| match me {
                    DescribedType::Concrete(_) => false,
                    DescribedType::Generic(me) => me == other,
                })
                .unwrap_or(false),
        }
    }

    pub fn as_defined(&self, generics: &HashMap<String, DescribedType>) -> Option<DescribedType> {
        match self {
            DescribedType::Concrete(me) => Some(DescribedType::Concrete(me.clone())),
            DescribedType::Generic(generic) => generics.get(generic).cloned(),
        }
    }
}

impl From<&DataType> for DescribedType {
    fn from(value: &DataType) -> Self {
        DescribedType::Concrete(value.clone())
    }
}

impl From<DataType> for DescribedType {
    fn from(value: DataType) -> Self {
        DescribedType::Concrete(value)
    }
}

impl Display for DescribedType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            DescribedType::Concrete(dt) => write!(f, "{}", dt),
            DescribedType::Generic(gen) => write!(f, "{}", gen),
        }
    }
}

#[derive(Clone, Hash, Debug)]
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
}

impl PartialEq for DataType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Vec(l0), Self::Vec(r0)) => l0 == r0,
            (Self::Option(l0), Self::Option(r0)) => l0 == r0,
            (Self::Undetermined, _) | (_, Self::Undetermined) => true,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl Display for DataType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            DataType::Undetermined => write!(f, "undetermined"),
            DataType::Void => write!(f, "void"),
            DataType::I8 => write!(f, "i8"),
            DataType::I16 => write!(f, "i16"),
            DataType::I32 => write!(f, "i32"),
            DataType::I64 => write!(f, "i64"),
            DataType::I128 => write!(f, "i128"),
            DataType::U8 => write!(f, "u8"),
            DataType::U16 => write!(f, "u16"),
            DataType::U32 => write!(f, "u32"),
            DataType::U64 => write!(f, "u64"),
            DataType::U128 => write!(f, "u128"),
            DataType::F32 => write!(f, "f32"),
            DataType::F64 => write!(f, "f64"),
            DataType::Bool => write!(f, "bool"),
            DataType::Byte => write!(f, "byte"),
            DataType::Char => write!(f, "char"),
            DataType::String => write!(f, "string"),
            DataType::Vec(dt) => write!(f, "Vec<{dt}>"),
            DataType::Option(dt) => write!(f, "Option<{dt}>"),
        }
    }
}
