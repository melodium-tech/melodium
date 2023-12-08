use super::{Attribuable, Attributes, DescribedType, Flow, Input};
use core::fmt::{Display, Formatter, Result};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Output {
    name: String,
    described_type: DescribedType,
    flow: Flow,
    attributes: Attributes,
}

impl Output {
    pub fn new(
        name: &str,
        described_type: DescribedType,
        flow: Flow,
        attributes: Attributes,
    ) -> Self {
        Self {
            name: name.to_string(),
            described_type,
            flow,
            attributes,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn described_type(&self) -> &DescribedType {
        &self.described_type
    }

    pub fn flow(&self) -> &Flow {
        &self.flow
    }

    /*
    pub fn matches_input(&self, input: &Input) -> bool {
        input.matches_output(self)
    }

    pub fn matches_output(&self, output: &Output) -> bool {
        &self.datatype == output.datatype() && &self.flow == output.flow()
    }*/
}

impl Attribuable for Output {
    fn attributes(&self) -> &Attributes {
        &self.attributes
    }
}

impl Display for Output {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self.flow {
            Flow::Block => {
                write!(f, "{}: Block<{}>", self.name, self.described_type)
            }
            Flow::Stream => {
                write!(f, "{}: Stream<{}>", self.name, self.described_type)
            }
        }
    }
}
