use crate::building::builder::get_value;
use crate::building::{
    BuildId, CheckBuildResult, CheckEnvironment, CheckStep, ContextualEnvironment,
    DynamicBuildResult, GenesisEnvironment, StaticBuildResult,
};
use crate::building::{Builder as BuilderTrait, HostTreatment};
use crate::design::Model;
use crate::error::LogicResult;
use crate::world::World;
use core::fmt::Debug;
use melodium_common::descriptor::{Model as ModelDescriptor, Parameterized};
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
        host_treatment: HostTreatment,
        host_build: Option<BuildId>,
        label: String,
        environment: &GenesisEnvironment,
    ) -> LogicResult<StaticBuildResult> {
        let mut environment = environment.clone();
        let mut remastered_environment = GenesisEnvironment::new();
        let descriptor = self.design.descriptor.upgrade().unwrap();

        // Assigning missing default values
        for (name, parameter) in descriptor
            .parameters()
            .iter()
            .filter(|(_, p)| p.default().is_some())
        {
            if !environment.variables().contains_key(name) {
                environment.add_variable(name, parameter.default().as_ref().unwrap().clone());
            }
        }

        // Assigning explicit data
        for (_, parameter) in self.design.parameters.iter() {
            let data = get_value(&parameter.value, &environment, None)
                .expect("Impossible data recoverage");

            remastered_environment.add_variable(&parameter.name, data.clone());
        }

        self.world
            .upgrade()
            .unwrap()
            .builder(descriptor.base_model().unwrap().identifier())
            .and_then(|builder| {
                builder.static_build(host_treatment, host_build, label, &remastered_environment)
            })
    }

    fn dynamic_build(
        &self,
        _build: BuildId,
        _with_inputs: Vec<String>,
        _environment: &ContextualEnvironment,
    ) -> Option<DynamicBuildResult> {
        // Doing nothing, models are not supposed to have dynamic building phase

        None
    }

    fn give_next(
        &self,
        _within_build: BuildId,
        _for_label: String,
        _for_outputs: Vec<String>,
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
