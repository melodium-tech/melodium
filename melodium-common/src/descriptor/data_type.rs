use crate::executive::Value;
use core::fmt::{Display, Formatter, Result};
use std::collections::HashMap;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
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

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct DataType {
    r#type: Type,
    structure: Structure,
}

impl DataType {
    pub fn new(structure: Structure, r#type: Type) -> Self {
        Self { structure, r#type }
    }

    pub fn structure(&self) -> &Structure {
        &self.structure
    }

    pub fn r#type(&self) -> &Type {
        &self.r#type
    }

    pub fn is_compatible(&self, value: &Value) -> bool {
        match &self.structure {
            Structure::Scalar => match &self.r#type {
                Type::Void => match value {
                    Value::Void(_) => true,
                    _ => false,
                },
                Type::I8 => match value {
                    Value::I8(_) => true,
                    _ => false,
                },
                Type::I16 => match value {
                    Value::I16(_) => true,
                    _ => false,
                },
                Type::I32 => match value {
                    Value::I32(_) => true,
                    _ => false,
                },
                Type::I64 => match value {
                    Value::I64(_) => true,
                    _ => false,
                },
                Type::I128 => match value {
                    Value::I128(_) => true,
                    _ => false,
                },
                Type::U8 => match value {
                    Value::U8(_) => true,
                    _ => false,
                },
                Type::U16 => match value {
                    Value::U16(_) => true,
                    _ => false,
                },
                Type::U32 => match value {
                    Value::U32(_) => true,
                    _ => false,
                },
                Type::U64 => match value {
                    Value::U64(_) => true,
                    _ => false,
                },
                Type::U128 => match value {
                    Value::U128(_) => true,
                    _ => false,
                },
                Type::F32 => match value {
                    Value::F32(_) => true,
                    _ => false,
                },
                Type::F64 => match value {
                    Value::F64(_) => true,
                    _ => false,
                },
                Type::Bool => match value {
                    Value::Bool(_) => true,
                    _ => false,
                },
                Type::Byte => match value {
                    Value::Byte(_) => true,
                    _ => false,
                },
                Type::Char => match value {
                    Value::Char(_) => true,
                    _ => false,
                },
                Type::String => match value {
                    Value::String(_) => true,
                    _ => false,
                },
            },

            Structure::Vector => match &self.r#type {
                Type::Void => match value {
                    Value::VecVoid(_) => true,
                    _ => false,
                },
                Type::I8 => match value {
                    Value::VecI8(_) => true,
                    _ => false,
                },
                Type::I16 => match value {
                    Value::VecI16(_) => true,
                    _ => false,
                },
                Type::I32 => match value {
                    Value::VecI32(_) => true,
                    _ => false,
                },
                Type::I64 => match value {
                    Value::VecI64(_) => true,
                    _ => false,
                },
                Type::I128 => match value {
                    Value::VecI128(_) => true,
                    _ => false,
                },
                Type::U8 => match value {
                    Value::VecU8(_) => true,
                    _ => false,
                },
                Type::U16 => match value {
                    Value::VecU16(_) => true,
                    _ => false,
                },
                Type::U32 => match value {
                    Value::VecU32(_) => true,
                    _ => false,
                },
                Type::U64 => match value {
                    Value::VecU64(_) => true,
                    _ => false,
                },
                Type::U128 => match value {
                    Value::VecU128(_) => true,
                    _ => false,
                },
                Type::F32 => match value {
                    Value::VecF32(_) => true,
                    _ => false,
                },
                Type::F64 => match value {
                    Value::VecF64(_) => true,
                    _ => false,
                },
                Type::Bool => match value {
                    Value::VecBool(_) => true,
                    _ => false,
                },
                Type::Byte => match value {
                    Value::VecByte(_) => true,
                    _ => false,
                },
                Type::Char => match value {
                    Value::VecChar(_) => true,
                    _ => false,
                },
                Type::String => match value {
                    Value::VecString(_) => true,
                    _ => false,
                },
            },
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Structure {
    Scalar,
    Vector,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Type {
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
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "{}",
            match self {
                Type::Void => "void",
                Type::I8 => "i8",
                Type::I16 => "i16",
                Type::I32 => "i32",
                Type::I64 => "i64",
                Type::I128 => "i128",
                Type::U8 => "u8",
                Type::U16 => "u16",
                Type::U32 => "u32",
                Type::U64 => "u64",
                Type::U128 => "u128",
                Type::F32 => "f32",
                Type::F64 => "f64",
                Type::Bool => "bool",
                Type::Byte => "byte",
                Type::Char => "char",
                Type::String => "string",
            }
        )
    }
}

impl Display for DataType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self.structure {
            Structure::Scalar => {
                write!(f, "{}", self.r#type)
            }
            Structure::Vector => {
                write!(f, "Vec<{}>", self.r#type)
            }
        }
    }
}
