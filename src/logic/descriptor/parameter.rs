
use super::datatype::DataType;

#[derive(Clone, PartialEq, Eq)]
pub struct Parameter {
    name: String,
    datatype: DataType,
}

impl Parameter {
    pub fn new(name: &str, datatype: DataType) -> Self {
        Self {
            name: name.to_string(),
            datatype
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn datatype(&self) -> &DataType {
        &self.datatype
    }
}
