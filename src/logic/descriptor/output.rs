
use super::datatype::DataType;

#[derive(Clone, PartialEq, Eq)]
pub struct Output {
    name: String,
    datatype: DataType,
}

impl Output {
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
