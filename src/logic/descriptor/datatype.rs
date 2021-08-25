
use crate::executive::value::Value;

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
                    Type::Boolean =>
                        match value { Value::Boolean(_) => true, _ => false },
                    Type::Integer =>
                        match value { Value::Integer(_) => true, _ => false },
                    Type::Real =>
                        match value { Value::Real(_) => true, _ => false },
                    Type::String =>
                        match value { Value::String(_) => true, _ => false },
                }
            },

            Structure::Vector => {

                match &self.r#type {
                    Type::Boolean =>
                        match value { Value::VecBoolean(_) => true, _ => false },
                    Type::Integer =>
                        match value { Value::VecInteger(_) => true, _ => false },
                    Type::Real =>
                        match value { Value::VecReal(_) => true, _ => false },
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
    Boolean,
    Integer,
    Real,
    String,
}
