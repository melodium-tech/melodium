
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
        todo!()
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
