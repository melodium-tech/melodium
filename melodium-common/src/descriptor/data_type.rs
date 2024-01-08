use super::DataTrait;
use core::fmt::{Display, Formatter, Result};

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

impl DataType {
    pub fn implements(&self, _data_trait: DataTrait) -> bool {
        false
    }
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
