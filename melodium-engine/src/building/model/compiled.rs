
use crate::building::Builder as BuilderTrait;
use melodium_common::descriptor::Treatment;
use melodium_common::executive::{Model, World as ExecutiveWorld};
use crate::error::LogicError;
use crate::building::{BuildId, ContextualEnvironment, GenesisEnvironment, StaticBuildResult, DynamicBuildResult, CheckBuildResult, CheckEnvironment, CheckStep};
use crate::world::World;
use std::sync::{Arc, Weak};
use core::fmt::Debug;


#[derive(Debug)]
pub struct Builder {
    world: Weak<World>,
    build_fn: fn(Arc<dyn ExecutiveWorld>) -> Arc<dyn Model>,
}

impl Builder {

    pub fn new(world: Weak<World>, build_fn: fn(Arc<dyn ExecutiveWorld>) -> Arc<dyn Model>) -> Self {
        Self {
            world,
            build_fn,
        }
    }
}

impl BuilderTrait for Builder {

    fn static_build(&self, _host_treatment: Option<Arc<dyn Treatment>>, _host_build: Option<BuildId>, _label: String, environment: &GenesisEnvironment) -> Result<StaticBuildResult, LogicError> {

        let world = self.world.upgrade().unwrap();
        let model = (self.build_fn)(world);

        for (name, value) in environment.variables() {
            model.set_parameter(name, value);
        }

        let id = world.add_model(Arc::clone(&model) as Arc<dyn Model>);

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


