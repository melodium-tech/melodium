
use std::fmt::*;
use std::sync::{Arc, Weak, RwLock};
use std::collections::HashMap;
use crate::logic::designer::ModelDesigner;
use super::identified::Identified;
use super::identifier::Identifier;
use super::parameterized::Parameterized;
use super::designable::Designable;
use super::buildable::Buildable;
use super::model::Model;
use super::core_model::CoreModel;
use super::parameter::Parameter;
use super::context::Context;
use super::super::builder::Builder;

#[derive(Debug)]
pub struct ConfiguredModel {
    identifier: Identifier,
    core_model: Arc<CoreModel>,
    parameters: HashMap<String, Parameter>,
    designer: RwLock<Option<Arc<RwLock<ModelDesigner>>>>,
    builder: RwLock<Option<Arc<Box<dyn Builder>>>>,
    auto_reference: Weak<Self>,
}

impl ConfiguredModel {
    pub fn new(identifier: Identifier, core_model: &Arc<CoreModel>) -> Self {
        Self {
            identifier,
            core_model: Arc::clone(core_model),
            parameters: HashMap::new(),
            designer: RwLock::new(None),
            builder: RwLock::new(None),
            auto_reference: Weak::default(),
        }
    }

    pub fn add_parameter(&mut self, parameter: Parameter) {
        self.parameters.insert(parameter.name().to_string(), parameter);
    }

    pub fn commit(self) -> Arc<Self> {
        Arc::new_cyclic(|me| Self {
            identifier: self.identifier,
            core_model: self.core_model,
            parameters: self.parameters,
            designer: self.designer,
            builder: self.builder,
            auto_reference: me.clone()
        })
    }

    pub fn set_designer(&self, designer: Arc<RwLock<ModelDesigner>>) {

        *self.designer.write().unwrap() = Some(designer);
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

    fn sources(&self) -> &HashMap<String, Vec<Arc<Context>>> {
        self.core_model.sources()
    }

    fn designer(&self) -> Option<Arc<RwLock<ModelDesigner>>> {
        Some(Arc::clone(self.designer.read().unwrap().as_ref().unwrap()))
    }
}

impl Display for ConfiguredModel {
    
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        writeln!(f, "Model `{}`", self.identifier.to_string())?;
        writeln!(f, "> `{}`", self.identifier.to_string())?;

        if !self.parameters.is_empty() {
            writeln!(f, "Parameters:")?;

            for parameter in &self.parameters {
                writeln!(f, "- {}", parameter.1)?;
            }
        }

        Ok(())
        
    }
}
