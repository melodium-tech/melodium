use super::{Attribuable, Attributes, DataType, Variability};
use crate::executive::Value;
use core::fmt::{Display, Formatter, Result};

#[derive(Clone, PartialEq, Debug)]
pub struct Parameter {
    name: String,
    variability: Variability,
    datatype: DataType,
    default: Option<Value>,
    attributes: Attributes,
}

impl Parameter {
    pub fn new(
        name: &str,
        variability: Variability,
        datatype: DataType,
        default: Option<Value>,
        attributes: Attributes,
    ) -> Self {
        Self {
            name: name.to_string(),
            variability,
            datatype,
            default,
            attributes,
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

impl Attribuable for Parameter {
    fn attributes(&self) -> &Attributes {
        &self.attributes
    }
}

impl Display for Parameter {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "{} {}: {}{}",
            self.variability,
            self.name,
            self.datatype,
            self.default
                .as_ref()
                .map(|d| format!(" = {}", d))
                .unwrap_or_default()
        )
    }
}
