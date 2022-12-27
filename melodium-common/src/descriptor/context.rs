
use core::fmt::{Display, Formatter, Result};
use std::collections::HashMap;
use std::iter::FromIterator;
use super::{DataType, Documented, Identified, Identifier};

#[derive(Debug)]
pub struct Context {
    identifier: Identifier,
    values: HashMap<String, DataType>,
    #[cfg(feature = "doc")]
    documentation: String,
}

impl Context {
    pub fn new(identifier: Identifier, values: Vec<(&str, DataType)>, documentation: String) -> Self {
        Self {
            identifier,
            values: HashMap::from_iter(values.iter().map(|v| (v.0.to_string(), v.1))),
            #[cfg(feature = "doc")]
            documentation,
        }
    }

    pub fn name(&self) -> &str {
        &self.identifier.name()
    }

    pub fn values(&self) -> &HashMap<String, DataType> {
        &self.values
    }
}

impl Documented for Context {
    fn documentation(&self) -> &str {
        #[cfg(feature = "doc")]
        {&self.documentation}
        #[cfg(not(feature = "doc"))]
        {&""}
    }
}

impl Identified for Context {
    fn identifier(&self) -> &Identifier {
        &self.identifier
    }
}

impl Display for Context {

    fn fmt(&self, f: &mut Formatter<'_>) -> Result {

        write!(f, "context {}", self.identifier.to_string())?;

        for (name, dt) in &self.values {
            write!(f, "{}: {}", name, dt)?
        }

        Ok(())
    }
}

