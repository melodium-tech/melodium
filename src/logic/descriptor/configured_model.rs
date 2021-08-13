
use std::sync::{Arc, Weak, RwLock};
use std::collections::HashMap;
use super::identified::Identified;
use super::identifier::Identifier;
use super::parameterized::Parameterized;
use super::designable::Designable;
use super::buildable::Buildable;
use super::model::Model;
use super::core_model::CoreModel;
use super::parameter::Parameter;
use super::super::builder::Builder;

#[derive(Debug)]
pub struct ConfiguredModel {
    identifier: Identifier,
    core_model: Arc<CoreModel>,
    parameters: HashMap<String, Parameter>,
    builder: RwLock<Option<Arc<Box<dyn Builder>>>>,
    auto_reference: Weak<Self>,
}

impl ConfiguredModel {
    pub fn new(identifier: Identifier, core_model: &Arc<CoreModel>) -> Self {
        Self {
            identifier,
            core_model: Arc::clone(core_model),
            parameters: HashMap::new(),
            builder: RwLock::new(None),
            auto_reference: Weak::new(),
        }
    }

    pub fn set_autoref(&mut self, reference: &Arc<Self>) {
        self.auto_reference = Arc::downgrade(reference);
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

impl Parameterized for ConfiguredModel {

    fn parameters(&self) -> &HashMap<String, Parameter> {
        &self.parameters
    }

    fn as_parameterized(&self) -> Arc<dyn Parameterized> {
        self.auto_reference.upgrade().unwrap()
    }
}

impl Designable for ConfiguredModel {
    
    fn register_builder(&self, builder: Box<dyn Builder>) {
        *(self.builder.write().unwrap()) = Some(Arc::new(builder))
    }
}

impl Buildable for ConfiguredModel {

    fn builder(&self) -> Arc<Box<dyn Builder>> {
        Arc::clone(self.builder.read().unwrap().as_ref().unwrap())
    }
}

impl Model for ConfiguredModel {

    fn is_core_model(&self) -> bool {
        false
    }

    fn core_model(&self) -> Arc<CoreModel> {
        Arc::clone(&self.core_model)
    }
}
