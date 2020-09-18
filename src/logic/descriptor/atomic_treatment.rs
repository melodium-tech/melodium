
use std::collections::HashMap;
use std::iter::FromIterator;
use super::identified::Identified;
use super::identifier::Identifier;
use super::input::Input;
use super::output::Output;
use super::parameter::Parameter;
use super::requirement::Requirement;
use super::treatment::Treatment;

pub struct AtomicTreatment {
    identifier: Identifier,
    parameters: HashMap<String, Parameter>,
    inputs: HashMap<String, Input>,
    outputs: HashMap<String, Output>,
}

impl AtomicTreatment {
    pub fn new(identifier: Identifier, parameters: Vec<Parameter>, inputs: Vec<Input>, outputs: Vec<Output>) -> Self {
        Self {
            identifier,
            parameters: HashMap::from_iter(parameters.iter().map(|p| (p.name().to_string(), p.clone()))),
            inputs: HashMap::from_iter(inputs.iter().map(|i| (i.name().to_string(), i.clone()))),
            outputs: HashMap::from_iter(outputs.iter().map(|o| (o.name().to_string(), o.clone()))),
        }
    }

}

impl Identified for AtomicTreatment {
    fn identifier(&self) -> &Identifier {
        &self.identifier
    }
}

impl Treatment for AtomicTreatment {

    fn inputs(&self) -> &HashMap<String, Input> {
        &self.inputs
    }

    fn outputs(&self) -> &HashMap<String, Output> {
        &self.outputs
    }

    fn parameters(&self) -> &HashMap<String, Parameter> {
        &self.parameters
    }

    fn requirements(&self) -> &HashMap<String, Requirement> {
        
        lazy_static! {
            static ref EMPTY_REQUIREMENTS: HashMap<String, Requirement> = HashMap::default();
        }

        &EMPTY_REQUIREMENTS
    }
}
