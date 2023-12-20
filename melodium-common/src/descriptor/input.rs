use super::{Attribuable, Attributes, DescribedType, Flow, Output};
use core::fmt::{Display, Formatter, Result};
use std::collections::HashMap;

#[derive(Clone, PartialEq, Debug)]
pub struct Input {
    name: String,
    described_type: DescribedType,
    flow: Flow,
    attributes: Attributes,
}

impl Input {
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

    pub fn matches_input(
        &self,
        generics: &HashMap<String, DescribedType>,
        input: &Input,
        other_generics: &HashMap<String, DescribedType>,
    ) -> bool {
        self.described_type
            .is_compatible(generics, input.described_type(), other_generics)
            && &self.flow == input.flow()
    }

    pub fn matches_output(
        &self,
        generics: &HashMap<String, DescribedType>,
        output: &Output,
        other_generics: &HashMap<String, DescribedType>,
    ) -> bool {
        self.described_type
            .is_compatible(generics, output.described_type(), other_generics)
            && &self.flow == output.flow()
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
                write!(f, "{}: Block<{}>", self.name, self.described_type)
            }
            Flow::Stream => {
                write!(f, "{}: Stream<{}>", self.name, self.described_type)
            }
        }
    }
}
