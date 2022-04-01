
use std::sync::{Arc, RwLock};
use super::*;
use super::super::error::LogicError;
use super::super::descriptor::model::Model;
use super::super::descriptor::buildable::Buildable;
use super::super::designer::ModelDesigner;
use super::super::descriptor::parameterized::Parameterized;
use super::super::super::executive::environment::{GenesisEnvironment, ContextualEnvironment};
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

        let borrowed_designer = self.designer.read().unwrap();

        // We do assign default values (will be replaced if some other explicitly assigned)
        for (_, declared_parameter) in borrowed_designer.descriptor().parameters() {

            if let Some(data) = declared_parameter.default() {
                remastered_environment.add_variable(declared_parameter.name(), data.clone());
            }
        }

        // Assigning explicit data
        for (_, parameter) in borrowed_designer.parameters().iter() {

            let borrowed_param = parameter.read().unwrap();

            let data = match borrowed_param.value().as_ref().unwrap() {
                Value::Raw(data) => data,
                Value::Variable(name) => {
                    if let Some(data) = environment.get_variable(&name) {
                        data
                    }
                    else {
                        borrowed_designer.descriptor().parameters().get(name).unwrap().default().as_ref().unwrap()
                    }
                },
                // Not possible in model to use context, should have been catcher by designed, aborting
                _ => panic!("Impossible data recoverage")
            };

            remastered_environment.add_variable(borrowed_param.name(), data.clone());
        }

        borrowed_designer.descriptor().core_model().builder().static_build(host_treatment, host_build, label, &remastered_environment)
    }

    fn dynamic_build(&self, _build: BuildId, _environment: &ContextualEnvironment) -> Option<DynamicBuildResult> {

        // Doing nothing, models are not supposed to have dynamic building phase

        None
    }

    fn give_next(&self, _within_build: BuildId, _for_label: String, _environment: &ContextualEnvironment) -> Option<DynamicBuildResult> {
        
        // Doing nothing, models are not supposed to have dynamic building phase
        
        None
    }

    fn check_dynamic_build(&self, _build: BuildId, _environment: CheckEnvironment, _previous_steps: Vec<CheckStep>) -> Option<CheckBuildResult> {

        // Doing nothing, models are not supposed to have dynamic building phase

        None
    }

    fn check_give_next(&self, _within_build: BuildId, _for_label: String, _environment: CheckEnvironment, _previous_steps: Vec<CheckStep>) -> Option<CheckBuildResult> {

        // Doing nothing, models are not supposed to have dynamic building phase

        None
    }
}
