
use std::sync::{Arc, Weak, RwLock};
use std::collections::HashMap;
use std::iter::FromIterator;
use super::identified::Identified;
use super::identifier::Identifier;
use super::parameterized::Parameterized;
use super::buildable::Buildable;
use super::model::Model;
use super::parameter::Parameter;
use super::super::builder::Builder;

#[derive(Debug)]
pub struct CoreModel {
    identifier: Identifier,
    parameters: HashMap<String, Parameter>,
    builder: Arc<Box<dyn Builder>>,
    auto_reference: RwLock<Weak<Self>>,
}

impl CoreModel {
    pub fn new(identifier: Identifier, parameters: Vec<Parameter>, builder: Box<dyn Builder>) -> Self {
        Self {
            identifier,
            parameters: HashMap::from_iter(parameters.iter().map(|p| (p.name().to_string(), p.clone()))),
            builder: Arc::new(builder),
            auto_reference: RwLock::new(Weak::new()),
        }
    }

    pub fn set_autoref(&self, reference: &Arc<Self>) {
        *self.auto_reference.write().unwrap() = Arc::downgrade(reference);
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

    fn as_parameterized(&self) -> Arc<dyn Parameterized> {
        self.auto_reference.read().unwrap().upgrade().unwrap()
    }
}

impl Buildable for CoreModel {

    fn builder(&self) -> Arc<Box<dyn Builder>> {
        Arc::clone(&self.builder)
    }
}

impl Model for CoreModel {

    fn is_core_model(&self) -> bool {
        true
    }

    fn core_model(&self) -> Arc<CoreModel> {
        self.auto_reference.read().unwrap().upgrade().unwrap()
    }
}
