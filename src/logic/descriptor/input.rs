
use super::datatype::DataType;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Input {
    name: String,
    datatype: DataType,
}

impl Input {
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
