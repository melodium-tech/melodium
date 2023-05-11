pub mod compiled;
pub mod designed;

pub use compiled::Builder as CompiledBuilder;
pub use designed::Builder as DesignedBuilder;

use super::Builder;
use crate::world::World;
use crate::{descriptor::Model, error::LogicResult};
use melodium_common::descriptor::{Buildable, ModelBuildMode, Status};
use std::sync::Arc;

pub fn get_builder(
    world: Arc<World>,
    descriptor: &Arc<dyn Buildable<ModelBuildMode>>,
) -> LogicResult<Arc<dyn Builder>> {
    match descriptor.build_mode() {
        ModelBuildMode::Compiled(build_fn) => Status::new_success(Arc::new(CompiledBuilder::new(
            Arc::downgrade(&world),
            build_fn,
        ))),
        ModelBuildMode::Designed() => Arc::clone(descriptor)
            .downcast_arc::<Model>()
            .unwrap()
            .design()
            .and_then(|design| {
                Status::new_success(
                    Arc::new(DesignedBuilder::new(Arc::downgrade(&world), design))
                        as Arc<dyn Builder>,
                )
            }),
    }
}
