
use std::collections::HashMap;
use std::iter::FromIterator;
use super::identified::Identified;
use super::identifier::Identifier;
use super::parameterized::Parameterized;
use super::model::Model;
use super::parameter::Parameter;

pub struct CoreModel {
    identifier: Identifier,
    parameters: HashMap<String, Parameter>,
}

impl CoreModel {
    pub fn new(identifier: Identifier, parameters: Vec<Parameter>) -> Self {
        Self {
            identifier,
            parameters: HashMap::from_iter(parameters.iter().map(|p| (p.name().to_string(), p.clone()))),
        }
    }
}

impl Identified for CoreModel {
    fn identifier(&self) -> &Identifier {
        &self.identifier
    }
}

impl Parameterized for CoreModel {
        
    fn parameters(&self) -> &HashMap<String, Parameter> {
        &self.parameters
    }
}

impl Model for CoreModel {

    fn is_core_model(&self) -> bool {
        true
    }

    fn core_model(&self) -> &CoreModel {
        &self
    }
}
