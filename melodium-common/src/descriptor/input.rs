use super::{DataType, Flow};
use core::fmt::{Display, Formatter, Result};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Input {
    name: String,
    datatype: DataType,
    flow: Flow,
}

impl Input {
    pub fn new(name: &str, datatype: DataType, flow: Flow) -> Self {
        Self {
            name: name.to_string(),
            datatype,
            flow,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn datatype(&self) -> &DataType {
        &self.datatype
    }

    pub fn flow(&self) -> &Flow {
        &self.flow
    }
}

impl Display for Input {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self.flow {
            Flow::Block => {
                write!(f, "{}: Block<{}>", self.name, self.datatype)
            }
            Flow::Stream => {
                write!(f, "{}: Stream<{}>", self.name, self.datatype)
            }
        }
    }
}
