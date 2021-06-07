
use super::datatype::DataType;
use crate::executive::value::Value;

#[derive(Clone, PartialEq, Debug)]
pub struct Parameter {
    name: String,
    datatype: DataType,
    default: Option<Value>,
}

impl Parameter {
    pub fn new(name: &str, datatype: DataType, default: Option<Value>) -> Self {
        Self {
            name: name.to_string(),
            datatype,
            default,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn datatype(&self) -> &DataType {
        &self.datatype
    }

    pub fn default(&self) -> &Option<Value> {
        &self.default
    }
}
