
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use super::*;
use super::super::error::LogicError;
use super::super::descriptor::model::Model;
use super::super::descriptor::buildable::Buildable;
use super::super::designer::ModelDesigner;
use super::super::descriptor::parameterized::Parameterized;
use super::super::super::executive::environment::{GenesisEnvironment, ContextualEnvironment};
use super::super::super::executive::model::Model as ExecutiveModel;
use super::super::super::executive::transmitter::Transmitter;
use super::super::super::executive::future::Future;
use super::super::descriptor::TreatmentDescriptor;
use super::super::designer::value::Value;

#[derive(Debug)]
pub struct ConfiguredModelBuilder {
    designer: Arc<RwLock<ModelDesigner>>
}

impl ConfiguredModelBuilder {
    pub fn new(designer: &Arc<RwLock<ModelDesigner>>) -> Self {
        Self {
            designer: Arc::clone(designer)
        }
    }
}

impl Builder for ConfiguredModelBuilder {

    fn static_build(&self, host_treatment: Option<Arc<dyn TreatmentDescriptor>>, host_build: Option<BuildId>, label: String, environment: &GenesisEnvironment) -> Result<StaticBuildResult, LogicError> {

        let mut remastered_environment = environment.base();

        // We do assign default values (will be replaced if some other explicitly assigned)
        for (_, declared_parameter) in self.designer.read().unwrap().descriptor().parameters() {

            if let Some(data) = declared_parameter.default() {
                remastered_environment.add_variable(declared_parameter.name(), data.clone());
            }
        }

        // Assigning explicit data
        for (_, parameter) in self.designer.read().unwrap().parameters().iter() {

            let borrowed_param = parameter.read().unwrap();

            let data = match borrowed_param.value().as_ref().unwrap() {
                Value::Raw(data) => data,
                Value::Variable(name) => {
                    environment.get_variable(&name).unwrap()
                },
                // Not possible in model to use context, should have been catcher by designed, aborting
                _ => panic!("Impossible data recoverage")
            };

            remastered_environment.add_variable(borrowed_param.name(), data.clone());
        }

        self.designer.read().unwrap().descriptor().core_model().builder().static_build(host_treatment, host_build, label, &remastered_environment)
    }

    fn dynamic_build(&self, build: BuildId, environment: &ContextualEnvironment) -> Option<DynamicBuildResult> {

        // Doing nothing, models are not supposed to have dynamic building phase

        None
    }

    fn give_next(&self, within_build: BuildId, for_label: String, environment: &ContextualEnvironment) -> Option<DynamicBuildResult> {
        
        // Doing nothing, models are not supposed to have dynamic building phase
        
        None
    }

    fn check_dynamic_build(&self, build: BuildId, ) -> Vec<LogicError> {
        // Doing nothing, models are not supposed to have dynamic building phase
        Vec::default()
    }

    fn check_give_next(&self, within_build: BuildId, for_label: String, ) -> Vec<LogicError> {
        // Doing nothing, models are not supposed to have dynamic building phase
        Vec::default()
    }
}
