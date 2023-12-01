use super::{Attribuable, Attributes, DataType, Flow, Output};
use core::fmt::{Display, Formatter, Result};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Input {
    name: String,
    datatype: DataType,
    flow: Flow,
    attributes: Attributes,
}

impl Input {
    pub fn new(name: &str, datatype: DataType, flow: Flow) -> Self {
        Self {
            name: name.to_string(),
            datatype,
            flow,
            attributes: Attributes::default(),
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

    pub fn matches_input(&self, input: &Input) -> bool {
        &self.datatype == input.datatype() && &self.flow == input.flow()
    }

    pub fn matches_output(&self, output: &Output) -> bool {
        &self.datatype == output.datatype() && &self.flow == output.flow()
    }
}

impl Attribuable for Input {
    fn attributes(&self) -> &Attributes {
        &self.attributes
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
