
//! Module for Type identification and structure semantic analysis.

use std::fmt;
use crate::ScriptError;
use crate::text::Type as TextType;
use melodium_common::descriptor::{DataType as DataTypeDescriptor, Structure as DataTypeStructureDescriptor, Type as DataTypeTypeDescriptor, Flow as FlowDescriptor};

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

        write!(f, "{}", match self {
            TypeFlow::Block => "Block",
            TypeFlow::Stream => "Stream",
        })
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

        write!(f, "{}", match self {
            TypeStructure::Scalar => "Scal",
            TypeStructure::Vector => "Vec",
        })
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
            _ => None
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

        write!(f, "{}", match self {
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
        })
    }
}

/// Structure managing and describing Type semantic analysis.
/// 
/// It owns the whole [text type](../../text/type/struct.Type.html).
/// 
/// Unlike most other elements of the semantic module, it does _not_ implement [Node trait](../common/trait.Node.html), because a type is considered as a property of its owner, not a children.
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
    /// # Example
    /// ```
    /// # use melodium::script::error::ScriptError;
    /// # use melodium::script::text::word::*;
    /// # use melodium::script::text::r#type::Type as TextType;
    /// # use melodium::script::semantic::r#type::*;
    /// let str_block_vec_int = "Vec<u64>";
    /// let str_block_scal_string = "string";
    /// let str_stream_vec_real = "Stream<Vec<f64>>";
    /// let str_stream_scal_bool = "Stream<bool>";
    /// 
    /// fn get_text_type(str: & str) -> TextType {
    ///     let words = get_words(str).unwrap();
    ///     let mut iter = words.iter();
    ///     TextType::build(&mut iter).unwrap()
    /// }
    /// 
    /// let type_block_vec_int = Type::new(get_text_type(str_block_vec_int))?;
    /// assert_eq!(type_block_vec_int.name, TypeName::U64);
    /// assert_eq!(type_block_vec_int.flow, TypeFlow::Block);
    /// assert_eq!(type_block_vec_int.structure, TypeStructure::Vector);
    /// 
    /// let type_block_scal_string = Type::new(get_text_type(str_block_scal_string))?;
    /// assert_eq!(type_block_scal_string.name, TypeName::String);
    /// assert_eq!(type_block_scal_string.flow, TypeFlow::Block);
    /// assert_eq!(type_block_scal_string.structure, TypeStructure::Scalar);
    /// 
    /// let type_stream_vec_real = Type::new(get_text_type(str_stream_vec_real))?;
    /// assert_eq!(type_stream_vec_real.name, TypeName::F64);
    /// assert_eq!(type_stream_vec_real.flow, TypeFlow::Stream);
    /// assert_eq!(type_stream_vec_real.structure, TypeStructure::Vector);
    /// 
    /// let type_stream_scal_bool = Type::new(get_text_type(str_stream_scal_bool))?;
    /// assert_eq!(type_stream_scal_bool.name, TypeName::Bool);
    /// assert_eq!(type_stream_scal_bool.flow, TypeFlow::Stream);
    /// assert_eq!(type_stream_scal_bool.structure, TypeStructure::Scalar);
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn new(text: TextType) -> Result<Self, ScriptError> {

        // Get type name.
        let name;
        if let Some(opt_name) = TypeName::from_string(text.name.string.as_ref()) {
            name = opt_name;
        }
        else {
            return Err(ScriptError::semantic("'".to_string() + &text.name.string + "' is not a valid type.", text.name.position))
        }

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

        let raw_structure = if valid_flow {&text.second_level_structure} else {&text.first_level_structure};
        let structure_name = match raw_structure.clone() {
            None => None,
            Some(s) => Some(s.string)
        };
        let structure = match structure_name.as_deref() {
            None => TypeStructure::Scalar,
            Some("Scal") => TypeStructure::Scalar,
            Some("Vec") => TypeStructure::Vector,
            _ => {
                return Err(ScriptError::semantic("'".to_string() + &structure_name.unwrap() + "' is not a valid structure.",
                raw_structure.as_ref().unwrap().position))
            }
        };

        Ok(Self{
            text,
            name,
            flow,
            structure,
        })
    }

    pub fn make_descriptor(&self) -> Result<(DataTypeDescriptor, FlowDescriptor), ScriptError> {

        let flow = match self.flow {
            TypeFlow::Block => FlowDescriptor::Block,
            TypeFlow::Stream => FlowDescriptor::Stream,
        };

        let structure = match self.structure {
            TypeStructure::Scalar => DataTypeStructureDescriptor::Scalar,
            TypeStructure::Vector => DataTypeStructureDescriptor::Vector,
        };

        let r#type = self.name.to_descriptor();

        Ok((DataTypeDescriptor::new(structure, r#type), flow))
    }
}


