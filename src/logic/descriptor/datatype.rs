
use crate::executive::value::Value;

macro_rules! datatype {
    ($data_structure:ident,$data_type:ident) => {
        crate::logic::descriptor::datatype::DataType::new(
            crate::logic::descriptor::datatype::Structure::$data_structure,
            crate::logic::descriptor::datatype::Type::$data_type,
        )
    };
}
pub(crate) use datatype;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct DataType {
    r#type: Type,
    structure: Structure,
}

impl DataType {
    pub fn new(structure: Structure, r#type: Type) -> Self {
        Self {
            structure,
            r#type
        }
    }

    pub fn structure(&self) -> &Structure {
        &self.structure
    }

    pub fn r#type(&self) -> &Type {
        &self.r#type
    }

    pub fn is_compatible(&self, value: &Value) -> bool {
        
        match &self.structure {

            Structure::Scalar => {

                match &self.r#type {
                    Type::I8 =>
                        match value { Value::I8(_) => true, _ => false },
                    Type::I16 =>
                        match value { Value::I16(_) => true, _ => false },
                    Type::I32 =>
                        match value { Value::I32(_) => true, _ => false },
                    Type::I64 =>
                        match value { Value::I64(_) => true, _ => false },
                    Type::I128 =>
                        match value { Value::I128(_) => true, _ => false },
                    Type::U8 =>
                        match value { Value::U8(_) => true, _ => false },
                    Type::U16 =>
                        match value { Value::U16(_) => true, _ => false },
                    Type::U32 =>
                        match value { Value::U32(_) => true, _ => false },
                    Type::U64 =>
                        match value { Value::U64(_) => true, _ => false },
                    Type::U128 =>
                        match value { Value::U128(_) => true, _ => false },
                    Type::F32 =>
                        match value { Value::F32(_) => true, _ => false },
                    Type::F64 =>
                        match value { Value::F64(_) => true, _ => false },
                    Type::Bool =>
                        match value { Value::Bool(_) => true, _ => false },
                    Type::Byte =>
                        match value { Value::Byte(_) => true, _ => false },
                    Type::Char =>
                        match value { Value::Char(_) => true, _ => false },
                    Type::String =>
                        match value { Value::String(_) => true, _ => false },
                }
            },

            Structure::Vector => {

                match &self.r#type {
                    Type::I8 =>
                        match value { Value::VecI8(_) => true, _ => false },
                    Type::I16 =>
                        match value { Value::VecI16(_) => true, _ => false },
                    Type::I32 =>
                        match value { Value::VecI32(_) => true, _ => false },
                    Type::I64 =>
                        match value { Value::VecI64(_) => true, _ => false },
                    Type::I128 =>
                        match value { Value::VecI128(_) => true, _ => false },
                    Type::U8 =>
                        match value { Value::VecU8(_) => true, _ => false },
                    Type::U16 =>
                        match value { Value::VecU16(_) => true, _ => false },
                    Type::U32 =>
                        match value { Value::VecU32(_) => true, _ => false },
                    Type::U64 =>
                        match value { Value::VecU64(_) => true, _ => false },
                    Type::U128 =>
                        match value { Value::VecU128(_) => true, _ => false },
                    Type::F32 =>
                        match value { Value::VecF32(_) => true, _ => false },
                    Type::F64 =>
                        match value { Value::VecF64(_) => true, _ => false },
                    Type::Bool =>
                        match value { Value::VecBool(_) => true, _ => false },
                    Type::Byte =>
                        match value { Value::VecByte(_) => true, _ => false },
                    Type::Char =>
                        match value { Value::VecChar(_) => true, _ => false },
                    Type::String =>
                        match value { Value::VecString(_) => true, _ => false },
                }
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
