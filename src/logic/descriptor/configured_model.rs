
use std::collections::HashMap;
use std::rc::Rc;
use intertrait::cast_to;
use super::identified::Identified;
use super::identifier::Identifier;
use super::parameterized::Parameterized;
use super::model::Model;
use super::core_model::CoreModel;
use super::parameter::Parameter;

#[derive(Debug)]
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

#[cast_to]
impl Identified for ConfiguredModel {
    fn identifier(&self) -> &Identifier {
        &self.identifier
    }
}

#[cast_to]
impl Parameterized for ConfiguredModel {

    fn parameters(&self) -> &HashMap<String, Parameter> {
        &self.parameters
    }
}

impl Model for ConfiguredModel {

    fn is_core_model(&self) -> bool {
        false
    }

    fn core_model(&self) -> Rc<CoreModel> {
        Rc::clone(&self.core_model)
    }
}
