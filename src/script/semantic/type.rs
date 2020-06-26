
//! Module for Type identification and structure semantic analysis.

use crate::script::error::ScriptError;
use crate::script::text::Type as TextType;

/// Enum for type structure identification.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum TypeStructure {
    /// Data is one unique value.
    Scalar,
    /// Data is a continuous one-dimension vector.
    Vector,
    /// Data is a matrix of values.
    Matrix,
    /// Data is a collection of one-dimensions vectors of a type, but all vectors are not required to be the same size.
    Collection,
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
    /// let str_vec_int = "Vec<Int>";
    /// let str_scal_string = "String";
    /// let str_col_real = "Col<Real>";
    /// let str_mat_bool = "Mat<Bool>";
    /// 
    /// fn get_text_type(str: & str) -> TextType {
    ///     let words = get_words(str).unwrap();
    ///     let mut iter = words.iter();
    ///     TextType::build(&mut iter).unwrap()
    /// }
    /// 
    /// let type_vec_int = Type::new(get_text_type(str_vec_int))?;
    /// assert_eq!(type_vec_int.name, TypeName::Integer);
    /// assert_eq!(type_vec_int.structure, TypeStructure::Vector);
    /// 
    /// let type_scal_string = Type::new(get_text_type(str_scal_string))?;
    /// assert_eq!(type_scal_string.name, TypeName::String);
    /// assert_eq!(type_scal_string.structure, TypeStructure::Scalar);
    /// 
    /// let type_col_real = Type::new(get_text_type(str_col_real))?;
    /// assert_eq!(type_col_real.name, TypeName::Real);
    /// assert_eq!(type_col_real.structure, TypeStructure::Collection);
    /// 
    /// let type_mat_bool = Type::new(get_text_type(str_mat_bool))?;
    /// assert_eq!(type_mat_bool.name, TypeName::Boolean);
    /// assert_eq!(type_mat_bool.structure, TypeStructure::Matrix);
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn new(text: TextType) -> Result<Self, ScriptError> {

        let name = match text.name.as_ref() {
            "Bool" => TypeName::Boolean,
            "Int" => TypeName::Integer,
            "Real" => TypeName::Real,
            "String" => TypeName::String,
            _ => {
                return Err(ScriptError::semantic("'".to_string() + &text.name + "' is not a valid type."))
            }
        };

        let structure = match text.structure.as_deref() {
            None => TypeStructure::Scalar,
            Some("Scal") => TypeStructure::Scalar,
            Some("Vec") => TypeStructure::Vector,
            Some("Mat") => TypeStructure::Matrix,
            Some("Col") => TypeStructure::Collection,
            _ => {
                return Err(ScriptError::semantic("'".to_string() + &text.name + "' is not a valid structure."))
            }
        };

        Ok(Self{
            text,
            name,
            structure,
        })
    }
}


