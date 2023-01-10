use crate::building::Builder as BuilderTrait;
use crate::building::{
    BuildId, CheckBuildResult, CheckEnvironment, CheckStep, ContextualEnvironment,
    DynamicBuildResult, GenesisEnvironment, StaticBuildResult,
};
use crate::design::{Model, Value};
use crate::error::LogicError;
use crate::world::World;
use core::fmt::Debug;
use melodium_common::descriptor::{Identified, Parameterized, Treatment};
use std::sync::{Arc, Weak};

#[derive(Debug)]
pub struct Builder {
    world: Weak<World>,
    design: Arc<Model>,
}

impl Builder {
    pub fn new(world: Weak<World>, design: Arc<Model>) -> Self {
        Self { world, design }
    }
}

impl BuilderTrait for Builder {
    fn static_build(
        &self,
        host_treatment: Option<Arc<dyn Treatment>>,
        host_build: Option<BuildId>,
        label: String,
        environment: &GenesisEnvironment,
    ) -> Result<StaticBuildResult, LogicError> {
        let mut remastered_environment = environment.base();
        let descriptor = self.design.descriptor.upgrade().unwrap();

        // We do assign default values (will be replaced if some other explicitly assigned)
        for (_, declared_parameter) in descriptor.parameters() {
            if let Some(data) = declared_parameter.default() {
                remastered_environment.add_variable(declared_parameter.name(), data.clone());
            }
        }

        // Assigning explicit data
        for (_, parameter) in self.design.parameters.iter() {
            let data = match &parameter.value {
                Value::Raw(data) => data,
                Value::Variable(name) => {
                    if let Some(data) = environment.get_variable(&name) {
                        data
                    } else {
                        descriptor
                            .parameters()
                            .get(name)
                            .unwrap()
                            .default()
                            .as_ref()
                            .unwrap()
                    }
                }
                // Not possible in model to use context, should have been catcher by designed, aborting
                _ => panic!("Impossible data recoverage"),
            };

            remastered_environment.add_variable(&parameter.name, data.clone());
        }

        self.world
            .upgrade()
            .unwrap()
            .builder(descriptor.identifier())?
            .static_build(host_treatment, host_build, label, &remastered_environment)
    }

    fn dynamic_build(
        &self,
        _build: BuildId,
        _environment: &ContextualEnvironment,
    ) -> Option<DynamicBuildResult> {
        // Doing nothing, models are not supposed to have dynamic building phase

        None
    }

    fn give_next(
        &self,
        _within_build: BuildId,
        _for_label: String,
        _environment: &ContextualEnvironment,
    ) -> Option<DynamicBuildResult> {
        // Doing nothing, models are not supposed to have dynamic building phase

        None
    }

    fn check_dynamic_build(
        &self,
        _build: BuildId,
        _environment: CheckEnvironment,
        _previous_steps: Vec<CheckStep>,
    ) -> Option<CheckBuildResult> {
        // Doing nothing, models are not supposed to have dynamic building phase

        None
    }

    fn check_give_next(
        &self,
        _within_build: BuildId,
        _for_label: String,
        _environment: CheckEnvironment,
        _previous_steps: Vec<CheckStep>,
    ) -> Option<CheckBuildResult> {
        // Doing nothing, models are not supposed to have dynamic building phase

        None
    }
}
