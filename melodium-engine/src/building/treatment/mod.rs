pub mod compiled;
pub mod designed;
pub mod source;

pub use compiled::Builder as CompiledBuilder;
pub use designed::Builder as DesignedBuilder;
pub use source::Builder as SourceBuilder;

use super::Builder;
use crate::world::World;
use crate::{descriptor::Treatment, error::LogicResult};
use melodium_common::descriptor::{Buildable, Status, TreatmentBuildMode};
use std::sync::Arc;

pub fn get_builder(
    world: Arc<World>,
    descriptor: &Arc<dyn Buildable<TreatmentBuildMode>>,
) -> LogicResult<Arc<dyn Builder>> {
    match descriptor.build_mode() {
        TreatmentBuildMode::Compiled(build_fn, desc) => Status::new_success(Arc::new(
            CompiledBuilder::new(Arc::downgrade(&world), desc, build_fn),
        )),
        TreatmentBuildMode::Source(desc) => {
            Status::new_success(Arc::new(SourceBuilder::new(Arc::downgrade(&world), desc)))
        }
        TreatmentBuildMode::Designed() => Arc::clone(descriptor)
            .downcast_arc::<Treatment>()
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
