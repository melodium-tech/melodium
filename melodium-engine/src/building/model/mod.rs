
pub mod compiled;
pub mod designed;

pub use compiled::Builder as CompiledBuilder;
pub use designed::Builder as DesignedBuilder;

use melodium_common::descriptor::{Buildable, ModelBuildMode};
use super::Builder;
use crate::world::World;
use crate::descriptor::Model;
use crate::error::LogicError;
use std::sync::Arc;

pub fn get_builder(world: Arc<World>, descriptor: &Arc<dyn Buildable<ModelBuildMode>>) -> Result<Arc<dyn Builder>, LogicError> {
    match descriptor.build_mode() {
        ModelBuildMode::Compiled(build_fn) => Ok(Arc::new(CompiledBuilder::new(Arc::downgrade(&world), build_fn))),
        ModelBuildMode::Designed() => {

            let designed_descriptor = Arc::clone(descriptor).downcast_arc::<Model>().unwrap();
            Ok(Arc::new(DesignedBuilder::new(Arc::downgrade(&world), designed_descriptor.design()?)))
        }
    }
}
