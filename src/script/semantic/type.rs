
use crate::script::error::ScriptError;
use crate::script::text::Type as TextType;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum TypeStructure {
    Scalar,
    Vector,
    Matrix,
    Collection,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum TypeName {
    Boolean,
    Integer,
    Real,
    String,
}

pub struct Type {
    pub text: TextType,

    pub name: TypeName,
    pub structure: TypeStructure,
}

impl Type {
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


