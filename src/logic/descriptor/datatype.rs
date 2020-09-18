
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
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
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum Structure {
    Scalar,
    Vector,
    Matrix,
    Collection,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Boolean,
    Integer,
    Real,
    String,
}
