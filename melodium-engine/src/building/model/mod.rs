
pub mod compiled;
pub mod designed;

pub use compiled::Builder as CompiledBuilder;
pub use designed::Builder as DesignedBuilder;

use melodium_common::descriptor::{Buildable, ModelBuildMode};
use super::Builder;
use crate::world::World;
use crate::descriptor::Model;
use std::sync::Arc;

pub fn get_builder(world: Arc<World>, descriptor: &Arc<dyn Buildable<ModelBuildMode>>) -> Arc<dyn Builder> {
    match descriptor.build_mode() {
        ModelBuildMode::Compiled(build_fn) => Arc::new(CompiledBuilder::new(Arc::downgrade(&world), build_fn)),
        ModelBuildMode::Designed(_) => {

            let designed_descriptor = descriptor.downcast_arc::<Model>().unwrap();
            Arc::new(DesignedBuilder::new(Arc::downgrade(&world), designed_descriptor.design().expect("No design available")))
        }
    }
}
