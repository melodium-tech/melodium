//! Module for Type identification and structure semantic analysis.

use crate::text::PositionnedString;
use crate::ScriptError;
use crate::{text::Type as TextType, ScriptResult};
use core::slice::Iter;
use melodium_common::descriptor::{
    DataType as DataTypeDescriptor, DescribedType as DescribedTypeDescriptor,
    Flow as FlowDescriptor,
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

/// Enum for type identification.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TypeContent {
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

    Option(Box<TypeContent>),
    Vec(Box<TypeContent>),

    Other(String),
}

impl TypeContent {
    fn from_positionned_strings(list: &Vec<PositionnedString>) -> Result<Self, ()> {
        Self::from_positionned_string(list.iter()).ok_or(())
    }

    fn from_positionned_string(mut iter: Iter<PositionnedString>) -> Option<Self> {
        let step = iter.next();
        if let Some(pos_str) = step {
            Some(match pos_str.string.as_str() {
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
                "Option" => Self::Option(Box::new(Self::from_positionned_string(iter)?)),
                "Vec" => Self::Vec(Box::new(Self::from_positionned_string(iter)?)),
                other => Self::Other(other.to_string()),
            })
        } else {
            None
        }
    }

    fn to_descriptor(&self) -> Result<DescribedTypeDescriptor, ()> {
        Ok(match self {
            Self::Other(name) => DescribedTypeDescriptor::Generic(name.clone()),
            me => DescribedTypeDescriptor::Concrete(me.to_datatype()?),
        })
    }

    fn to_datatype(&self) -> Result<DataTypeDescriptor, ()> {
        match self {
            Self::Void => Ok(DataTypeDescriptor::Void),
            Self::I8 => Ok(DataTypeDescriptor::I8),
            Self::I16 => Ok(DataTypeDescriptor::I16),
            Self::I32 => Ok(DataTypeDescriptor::I32),
            Self::I64 => Ok(DataTypeDescriptor::I64),
            Self::I128 => Ok(DataTypeDescriptor::I128),
            Self::U8 => Ok(DataTypeDescriptor::U8),
            Self::U16 => Ok(DataTypeDescriptor::U16),
            Self::U32 => Ok(DataTypeDescriptor::U32),
            Self::U64 => Ok(DataTypeDescriptor::U64),
            Self::U128 => Ok(DataTypeDescriptor::U128),
            Self::F32 => Ok(DataTypeDescriptor::F32),
            Self::F64 => Ok(DataTypeDescriptor::F64),
            Self::Bool => Ok(DataTypeDescriptor::Bool),
            Self::Byte => Ok(DataTypeDescriptor::Byte),
            Self::Char => Ok(DataTypeDescriptor::Char),
            Self::String => Ok(DataTypeDescriptor::String),
            Self::Option(internal) => Ok(DataTypeDescriptor::Option(Box::new(
                internal.to_datatype()?,
            ))),
            Self::Vec(internal) => Ok(DataTypeDescriptor::Vec(Box::new(internal.to_datatype()?))),
            Self::Other(_) => Err(()),
        }
    }
}

impl fmt::Display for TypeContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeContent::Void => write!(f, "void"),
            TypeContent::I8 => write!(f, "i8"),
            TypeContent::I16 => write!(f, "i16"),
            TypeContent::I32 => write!(f, "i32"),
            TypeContent::I64 => write!(f, "i64"),
            TypeContent::I128 => write!(f, "i128"),
            TypeContent::U8 => write!(f, "u8"),
            TypeContent::U16 => write!(f, "u16"),
            TypeContent::U32 => write!(f, "u32"),
            TypeContent::U64 => write!(f, "u64"),
            TypeContent::U128 => write!(f, "u128"),
            TypeContent::F32 => write!(f, "f32"),
            TypeContent::F64 => write!(f, "f64"),
            TypeContent::Bool => write!(f, "bool"),
            TypeContent::Byte => write!(f, "byte"),
            TypeContent::Char => write!(f, "char"),
            TypeContent::String => write!(f, "string"),
            TypeContent::Option(inner) => write!(f, "Option<{inner}>"),
            TypeContent::Vec(inner) => write!(f, "Vec<{inner}>"),
            TypeContent::Other(name) => write!(f, "{name}"),
        }
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

    pub flow: TypeFlow,
    pub content: TypeContent,
}

impl Type {
    /// Create a new semantic type, based on textual type.
    ///
    /// * `text`: the textual type.
    ///
    pub fn new(text: TextType) -> ScriptResult<Self> {
        let mut remove_first = false;
        let flow = text
            .level_structure
            .first()
            .map(|possible_flow| match possible_flow.string.as_str() {
                "Block" => {
                    remove_first = true;
                    TypeFlow::Block
                }
                "Stream" => {
                    remove_first = true;
                    TypeFlow::Stream
                }
                _ => TypeFlow::Block,
            })
            .unwrap_or(TypeFlow::Block);

        let mut level_structure = text.level_structure.clone();
        if remove_first {
            level_structure.remove(0);
        }

        match TypeContent::from_positionned_strings(&level_structure)
            .map_err(|_| ScriptError::invalid_structure(177, text.name.clone()))
        {
            Ok(content) => ScriptResult::new_success(Self {
                content,
                text,
                flow,
            }),
            Err(error) => ScriptResult::new_failure(error),
        }
    }

    pub fn make_descriptor(&self) -> ScriptResult<(DescribedTypeDescriptor, FlowDescriptor)> {
        let flow = match self.flow {
            TypeFlow::Block => FlowDescriptor::Block,
            TypeFlow::Stream => FlowDescriptor::Stream,
        };

        match self
            .content
            .to_descriptor()
            .map_err(|_| ScriptError::unsupported_nested_generic(176, self.text.name.clone()))
        {
            Ok(described_type) => ScriptResult::new_success((described_type, flow)),
            Err(error) => ScriptResult::new_failure(error),
        }
    }
}
