//! Module for Type identification and structure semantic analysis.

use crate::ScriptError;
use crate::{text::Type as TextType, ScriptResult};
use melodium_common::descriptor::{
    DataType as DataTypeDescriptor, Flow as FlowDescriptor,
    Structure as DataTypeStructureDescriptor, Type as DataTypeTypeDescriptor,
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
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
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
}

impl TypeName {
    fn from_string(name: &str) -> Option<Self> {
        match name {
            "void" => Some(Self::Void),
            "i8" => Some(Self::I8),
            "i16" => Some(Self::I16),
            "i32" => Some(Self::I32),
            "i64" => Some(Self::I64),
            "i128" => Some(Self::I128),
            "u8" => Some(Self::U8),
            "u16" => Some(Self::U16),
            "u32" => Some(Self::U32),
            "u64" => Some(Self::U64),
            "u128" => Some(Self::U128),
            "f32" => Some(Self::F32),
            "f64" => Some(Self::F64),
            "bool" => Some(Self::Bool),
            "byte" => Some(Self::Byte),
            "char" => Some(Self::Char),
            "string" => Some(Self::String),
            _ => None,
        }
    }

    fn to_descriptor(&self) -> DataTypeTypeDescriptor {
        match self {
            Self::Void => DataTypeTypeDescriptor::Void,
            Self::I8 => DataTypeTypeDescriptor::I8,
            Self::I16 => DataTypeTypeDescriptor::I16,
            Self::I32 => DataTypeTypeDescriptor::I32,
            Self::I64 => DataTypeTypeDescriptor::I64,
            Self::I128 => DataTypeTypeDescriptor::I128,
            Self::U8 => DataTypeTypeDescriptor::U8,
            Self::U16 => DataTypeTypeDescriptor::U16,
            Self::U32 => DataTypeTypeDescriptor::U32,
            Self::U64 => DataTypeTypeDescriptor::U64,
            Self::U128 => DataTypeTypeDescriptor::U128,
            Self::F32 => DataTypeTypeDescriptor::F32,
            Self::F64 => DataTypeTypeDescriptor::F64,
            Self::Bool => DataTypeTypeDescriptor::Bool,
            Self::Byte => DataTypeTypeDescriptor::Byte,
            Self::Char => DataTypeTypeDescriptor::Char,
            Self::String => DataTypeTypeDescriptor::String,
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

        if let Some(name) = TypeName::from_string(text.name.string.as_ref()) {
            ScriptResult::new_success(Self {
                text,
                name,
                flow,
                structure,
            })
        } else {
            ScriptResult::new_failure(ScriptError::invalid_type(109, text.name.clone()))
        }
    }

    pub fn make_descriptor(&self) -> ScriptResult<(DataTypeDescriptor, FlowDescriptor)> {
        let flow = match self.flow {
            TypeFlow::Block => FlowDescriptor::Block,
            TypeFlow::Stream => FlowDescriptor::Stream,
        };

        let structure = match self.structure {
            TypeStructure::Scalar => DataTypeStructureDescriptor::Scalar,
            TypeStructure::Vector => DataTypeStructureDescriptor::Vector,
        };

        let r#type = self.name.to_descriptor();

        ScriptResult::new_success((DataTypeDescriptor::new(structure, r#type), flow))
    }
}
