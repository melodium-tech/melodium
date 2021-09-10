
//! Module for Type identification and structure semantic analysis.

use crate::script::error::ScriptError;
use crate::script::text::Type as TextType;
use crate::logic::descriptor::{DataTypeDescriptor, DataTypeStructureDescriptor, DataTypeTypeDescriptor, FlowDescriptor};

/// Enum for type flow identification.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum TypeFlow {
    /// Data flow is blocking.
    Block,
    /// Data flow is a stream.
    Stream,
}

/// Enum for type structure identification.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum TypeStructure {
    /// Data is one unique value.
    Scalar,
    /// Data is a continuous one-dimension vector.
    Vector,
}

/// Enum for type identification.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum TypeName {
    /// Value is either `true` or `false`.
    Boolean,
    /// Value is any positive or negative integer number.
    Integer,
    /// Value is any positive or negative real number.
    Real,
    /// Value is any string of characters.
    String,
}

/// Structure managing and describing Type semantic analysis.
/// 
/// It owns the whole [text type](../../text/type/struct.Type.html).
/// 
/// Unlike most other elements of the semantic module, it does _not_ implement [Node trait](../common/trait.Node.html), because a type is considered as a property of its owner, not a children.
/// Also, it is a build-in element of Mélodium language so don't have any references to manage.
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
    /// # use melodium_rust::script::error::ScriptError;
    /// # use melodium_rust::script::text::word::*;
    /// # use melodium_rust::script::text::r#type::Type as TextType;
    /// # use melodium_rust::script::semantic::r#type::*;
    /// let str_block_vec_int = "Vec<Int>";
    /// let str_block_scal_string = "String";
    /// let str_stream_vec_real = "Stream<Vec<Real>>";
    /// let str_stream_scal_bool = "Stream<Bool>";
    /// 
    /// fn get_text_type(str: & str) -> TextType {
    ///     let words = get_words(str).unwrap();
    ///     let mut iter = words.iter();
    ///     TextType::build(&mut iter).unwrap()
    /// }
    /// 
    /// let type_block_vec_int = Type::new(get_text_type(str_block_vec_int))?;
    /// assert_eq!(type_block_vec_int.name, TypeName::Integer);
    /// assert_eq!(type_block_vec_int.flow, TypeFlow::Block);
    /// assert_eq!(type_block_vec_int.structure, TypeStructure::Vector);
    /// 
    /// let type_block_scal_string = Type::new(get_text_type(str_block_scal_string))?;
    /// assert_eq!(type_block_scal_string.name, TypeName::String);
    /// assert_eq!(type_block_scal_string.flow, TypeFlow::Block);
    /// assert_eq!(type_block_scal_string.structure, TypeStructure::Scalar);
    /// 
    /// let type_stream_vec_real = Type::new(get_text_type(str_stream_vec_real))?;
    /// assert_eq!(type_stream_vec_real.name, TypeName::Real);
    /// assert_eq!(type_stream_vec_real.flow, TypeFlow::Stream);
    /// assert_eq!(type_stream_vec_real.structure, TypeStructure::Vector);
    /// 
    /// let type_stream_scal_bool = Type::new(get_text_type(str_stream_scal_bool))?;
    /// assert_eq!(type_stream_scal_bool.name, TypeName::Boolean);
    /// assert_eq!(type_stream_scal_bool.flow, TypeFlow::Stream);
    /// assert_eq!(type_stream_scal_bool.structure, TypeStructure::Scalar);
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn new(text: TextType) -> Result<Self, ScriptError> {

        let name = match text.name.string.as_ref() {
            "Bool" => TypeName::Boolean,
            "Int" => TypeName::Integer,
            "Real" => TypeName::Real,
            "String" => TypeName::String,
            _ => {
                return Err(ScriptError::semantic("'".to_string() + &text.name.string + "' is not a valid type.", text.name.position))
            }
        };

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

        let r#type = match self.name {
            TypeName::Boolean => DataTypeTypeDescriptor::Boolean,
            TypeName::Integer => DataTypeTypeDescriptor::Integer,
            TypeName::Real => DataTypeTypeDescriptor::Real,
            TypeName::String => DataTypeTypeDescriptor::String,
        };

        Ok((DataTypeDescriptor::new(structure, r#type), flow))
    }
}


