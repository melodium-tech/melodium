
use std::rc::{Rc, Weak};
use std::collections::HashMap;
use std::iter::FromIterator;
use super::identified::Identified;
use super::identifier::Identifier;
use super::parameterized::Parameterized;
use super::model::Model;
use super::parameter::Parameter;

#[derive(Debug)]
pub struct CoreModel {
    identifier: Identifier,
    parameters: HashMap<String, Parameter>,
    auto_reference: Weak<Self>,
}

impl CoreModel {
    pub fn new(identifier: Identifier, parameters: Vec<Parameter>) -> Self {
        Self {
            identifier,
            parameters: HashMap::from_iter(parameters.iter().map(|p| (p.name().to_string(), p.clone()))),
            auto_reference: Weak::new(),
        }
    }

    pub fn set_autoref(&mut self, reference: &Rc<Self>) {
        self.auto_reference = Rc::downgrade(reference);
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

    fn core_model(&self) -> Rc<CoreModel> {
        self.auto_reference.upgrade().unwrap()
    }
}
