
use std::collections::HashMap;
use std::rc::Rc;
use super::identified::Identified;
use super::identifier::Identifier;
use super::model::Model;
use super::core_model::CoreModel;
use super::parameter::Parameter;

pub struct ConfiguredModel {
    identifier: Identifier,
    core_model: Rc<CoreModel>,
    parameters: HashMap<String, Parameter>,
}

impl ConfiguredModel {
    pub fn new(identifier: Identifier, core_model: &Rc<CoreModel>) -> Self {
        Self {
            identifier,
            core_model: Rc::clone(core_model),
            parameters: HashMap::new(),
        }
    }

    pub fn add_parameter(&mut self, parameter: Parameter) {
        self.parameters.insert(parameter.name().to_string(), parameter);
    }
}

impl Identified for ConfiguredModel {
    fn identifier(&self) -> &Identifier {
        &self.identifier
    }
}

impl Model for ConfiguredModel {
    
    fn parameters(&self) -> &HashMap<String, Parameter> {
        &self.parameters
    }

    fn is_core_model(&self) -> bool {
        false
    }

    fn core_model(&self) -> &CoreModel {
        &self.core_model
    }
}
