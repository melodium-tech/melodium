
use core::fmt::{Display, Formatter, Result};
use crate::executive::Value;
use super::{DataType, Variability};

#[derive(Clone, PartialEq, Debug)]
pub struct Parameter {
    name: String,
    variability: Variability,
    datatype: DataType,
    default: Option<Value>,
}

impl Parameter {
    pub fn new(name: &str, variability: Variability, datatype: DataType, default: Option<Value>) -> Self {
        Self {
            name: name.to_string(),
            variability,
            datatype,
            default,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn variability(&self) -> &Variability {
        &self.variability
    }

    pub fn datatype(&self) -> &DataType {
        &self.datatype
    }

    pub fn default(&self) -> &Option<Value> {
        &self.default
    }
}

impl Display for Parameter {

    fn fmt(&self, f: &mut Formatter<'_>) -> Result {

        write!(f, "{} {}: {}{}",
            self.variability,
            self.name,
            self.datatype,
            self.default.as_ref().map(|d| format!(" = {}", d)).unwrap_or_default()
        )
    }
}
