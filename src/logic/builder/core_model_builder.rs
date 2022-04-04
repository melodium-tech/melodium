
use crate::executive::world::World;
use std::fmt::Debug;
use std::sync::Arc;
use crate::logic::error::LogicError;
use crate::logic::builder::*;
use crate::logic::descriptor::TreatmentDescriptor;
use crate::executive::environment::{ContextualEnvironment, GenesisEnvironment};
use crate::executive::model::Model;

#[derive(Debug)]
pub struct CoreModelBuilder {
    new_model: fn(Arc<World>) -> Arc<dyn Model>,
}

impl CoreModelBuilder {

    pub fn new(new_model: fn(Arc<World>) -> Arc<dyn Model>) -> Self {
        Self {
            new_model
        }
    }
}

impl Builder for CoreModelBuilder {

    fn static_build(&self, _host_treatment: Option<Arc<dyn TreatmentDescriptor>>, _host_build: Option<BuildId>, _label: String, environment: &GenesisEnvironment) -> Result<StaticBuildResult, LogicError> {

        let model = (self.new_model)(environment.world());

        for (name, value) in environment.variables() {
            model.set_parameter(name, value);
        }

        let id = environment.register_model(Arc::clone(&model) as Arc<dyn Model>);

        model.set_id(id);
        
        Ok(StaticBuildResult::Model(model))
    }

    fn dynamic_build(&self, _build: BuildId, _environment: &ContextualEnvironment) -> Option<DynamicBuildResult> {
        None
    }

    fn give_next(&self, _within_build: BuildId, _for_label: String, _environment: &ContextualEnvironment) -> Option<DynamicBuildResult> {
        None
    }

    fn check_dynamic_build(&self, _build: BuildId, _environment: CheckEnvironment, _previous_steps: Vec<CheckStep>) -> Option<CheckBuildResult> {
        None
    }

    fn check_give_next(&self, _within_build: BuildId, _for_label: String, _environment: CheckEnvironment, _previous_steps: Vec<CheckStep>) -> Option<CheckBuildResult> {
        None
    }
}



