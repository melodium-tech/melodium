//! Module for Type identification and structure semantic analysis.

use crate::ScriptError;
use crate::{text::Type as TextType, ScriptResult};
use melodium_common::descriptor::{
    DataType as DataTypeDescriptor, DescribedType as DescribedTypeDescriptor,
    Flow as FlowDescriptor, Structure as DataTypeStructureDescriptor,
    Type as DataTypeTypeDescriptor,
};
use std::fmt;

/// Enum for type flow identification.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum TypeFlow {
    /// Data flow is blocking.
    Block,
    /// Data flow is a stream.
    Stream,
}

impl fmt::Display for TypeFlow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TypeFlow::Block => "Block",
                TypeFlow::Stream => "Stream",
            }
        )
    }
}

/// Enum for type structure identification.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum TypeStructure {
    /// Data is one unique value.
    Scalar,
    /// Data is a continuous one-dimension vector.
    Vector,
}

impl fmt::Display for TypeStructure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TypeStructure::Scalar => "Scal",
                TypeStructure::Vector => "Vec",
            }
        )
    }
}

/// Enum for type identification.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TypeName {
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

    Other(String),
}

impl TypeName {
    fn from_string(name: &str) -> Self {
        match name {
            "void" => Self::Void,
            "i8" => Self::I8,
            "i16" => Self::I16,
            "i32" => Self::I32,
            "i64" => Self::I64,
            "i128" => Self::I128,
            "u8" => Self::U8,
            "u16" => Self::U16,
            "u32" => Self::U32,
            "u64" => Self::U64,
            "u128" => Self::U128,
            "f32" => Self::F32,
            "f64" => Self::F64,
            "bool" => Self::Bool,
            "byte" => Self::Byte,
            "char" => Self::Char,
            "string" => Self::String,
            other => Self::Other(other.to_string()),
        }
    }

    fn to_descriptor(&self, structure: DataTypeStructureDescriptor) -> DescribedTypeDescriptor {
        match self {
            Self::Void => DescribedTypeDescriptor::Concrete(DataTypeDescriptor::new(
                structure,
                DataTypeTypeDescriptor::Void,
            )),
            Self::I8 => DescribedTypeDescriptor::Concrete(DataTypeDescriptor::new(
                structure,
                DataTypeTypeDescriptor::I8,
            )),
            Self::I16 => DescribedTypeDescriptor::Concrete(DataTypeDescriptor::new(
                structure,
                DataTypeTypeDescriptor::I16,
            )),
            Self::I32 => DescribedTypeDescriptor::Concrete(DataTypeDescriptor::new(
                structure,
                DataTypeTypeDescriptor::I32,
            )),
            Self::I64 => DescribedTypeDescriptor::Concrete(DataTypeDescriptor::new(
                structure,
                DataTypeTypeDescriptor::I64,
            )),
            Self::I128 => DescribedTypeDescriptor::Concrete(DataTypeDescriptor::new(
                structure,
                DataTypeTypeDescriptor::I128,
            )),
            Self::U8 => DescribedTypeDescriptor::Concrete(DataTypeDescriptor::new(
                structure,
                DataTypeTypeDescriptor::U8,
            )),
            Self::U16 => DescribedTypeDescriptor::Concrete(DataTypeDescriptor::new(
                structure,
                DataTypeTypeDescriptor::U16,
            )),
            Self::U32 => DescribedTypeDescriptor::Concrete(DataTypeDescriptor::new(
                structure,
                DataTypeTypeDescriptor::U32,
            )),
            Self::U64 => DescribedTypeDescriptor::Concrete(DataTypeDescriptor::new(
                structure,
                DataTypeTypeDescriptor::U64,
            )),
            Self::U128 => DescribedTypeDescriptor::Concrete(DataTypeDescriptor::new(
                structure,
                DataTypeTypeDescriptor::U128,
            )),
            Self::F32 => DescribedTypeDescriptor::Concrete(DataTypeDescriptor::new(
                structure,
                DataTypeTypeDescriptor::F32,
            )),
            Self::F64 => DescribedTypeDescriptor::Concrete(DataTypeDescriptor::new(
                structure,
                DataTypeTypeDescriptor::F64,
            )),
            Self::Bool => DescribedTypeDescriptor::Concrete(DataTypeDescriptor::new(
                structure,
                DataTypeTypeDescriptor::Bool,
            )),
            Self::Byte => DescribedTypeDescriptor::Concrete(DataTypeDescriptor::new(
                structure,
                DataTypeTypeDescriptor::Byte,
            )),
            Self::Char => DescribedTypeDescriptor::Concrete(DataTypeDescriptor::new(
                structure,
                DataTypeTypeDescriptor::Char,
            )),
            Self::String => DescribedTypeDescriptor::Concrete(DataTypeDescriptor::new(
                structure,
                DataTypeTypeDescriptor::String,
            )),
            Self::Other(name) => DescribedTypeDescriptor::Generic(name.clone()),
        }
    }
}

impl fmt::Display for TypeName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TypeName::Void => "void",
                TypeName::I8 => "i8",
                TypeName::I16 => "i16",
                TypeName::I32 => "i32",
                TypeName::I64 => "i64",
                TypeName::I128 => "i128",
                TypeName::U8 => "u8",
                TypeName::U16 => "u16",
                TypeName::U32 => "u32",
                TypeName::U64 => "u64",
                TypeName::U128 => "u128",
                TypeName::F32 => "f32",
                TypeName::F64 => "f64",
                TypeName::Bool => "bool",
                TypeName::Byte => "byte",
                TypeName::Char => "char",
                TypeName::String => "string",
                TypeName::Other(name) => name.as_str(),
            }
        )
    }
}

/// Structure managing and describing Type semantic analysis.
///
/// It owns the whole [text type](TextType).
///
/// Unlike most other elements of the semantic module, it does _not_ implement [Node trait](super::Node), because a type is considered as a property of its owner, not a children.
/// Also, it is a build-in element of Mélodium language so don't have any references to manage.
#[derive(Debug)]
pub struct Type {
    pub text: TextType,

    pub name: TypeName,
    pub flow: TypeFlow,
    pub structure: TypeStructure,
}

impl Type {
    /// Create a new semantic type, based on textual type.
    ///
    /// * `text`: the textual type.
    ///
    pub fn new(text: TextType) -> ScriptResult<Self> {
        // Keep if flow has been specified.
        let mut valid_flow = true;
        let flow_name = match text.first_level_structure.clone() {
            None => None,
            Some(s) => Some(s.string),
        };
        let flow = match flow_name.as_deref() {
            None => TypeFlow::Block,
            Some("Block") => TypeFlow::Block,
            Some("Stream") => TypeFlow::Stream,
            _ => {
                valid_flow = false;
                TypeFlow::Block
            }
        };

        let raw_structure = if valid_flow {
            &text.second_level_structure
        } else {
            &text.first_level_structure
        };
        let structure_name = match raw_structure.clone() {
            None => None,
            Some(s) => Some(s.string),
        };
        let structure = match structure_name.as_deref() {
            None => TypeStructure::Scalar,
            Some("Scal") => TypeStructure::Scalar,
            Some("Vec") => TypeStructure::Vector,
            _ => {
                return ScriptResult::new_failure(ScriptError::invalid_structure(
                    110,
                    raw_structure.as_ref().unwrap().clone(),
                ));
            }
        };

        ScriptResult::new_success(Self {
            name: TypeName::from_string(text.name.string.as_ref()),
            text,
            flow,
            structure,
        })
    }

    pub fn make_descriptor(&self) -> ScriptResult<(DescribedTypeDescriptor, FlowDescriptor)> {
        let flow = match self.flow {
            TypeFlow::Block => FlowDescriptor::Block,
            TypeFlow::Stream => FlowDescriptor::Stream,
        };

        let structure = match self.structure {
            TypeStructure::Scalar => DataTypeStructureDescriptor::Scalar,
            TypeStructure::Vector => DataTypeStructureDescriptor::Vector,
        };

        let r#type = self.name.to_descriptor(structure);

        ScriptResult::new_success((r#type, flow))
    }
}
