use super::{Attribuable, Attributes, DescribedType, Variability};
use crate::executive::Value;
use core::fmt::{Display, Formatter, Result};

#[derive(Clone, PartialEq, Debug)]
pub struct Parameter {
    name: String,
    variability: Variability,
    described_type: DescribedType,
    default: Option<Value>,
    attributes: Attributes,
}

impl Parameter {
    pub fn new(
        name: &str,
        variability: Variability,
        described_type: DescribedType,
        default: Option<Value>,
        attributes: Attributes,
    ) -> Self {
        Self {
            name: name.to_string(),
            variability,
            described_type,
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

    pub fn described_type(&self) -> &DescribedType {
        &self.described_type
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
            self.described_type,
            self.default
                .as_ref()
                .map(|d| format!(" = {}", d))
                .unwrap_or_default()
        )
    }
}
