
pub mod compiled;
pub mod source;
pub mod designed;

pub use compiled::Builder as CompiledBuilder;
pub use source::Builder as SourceBuilder;
pub use designed::Builder as DesignedBuilder;

use melodium_common::descriptor::{Buildable, TreatmentBuildMode};
use super::Builder;
use crate::world::World;
use crate::descriptor::Treatment;
use crate::error::LogicError;
use std::sync::Arc;

pub fn get_builder(world: Arc<World>, descriptor: &Arc<dyn Buildable<TreatmentBuildMode>>) -> Result<Arc<dyn Builder>, LogicError> {
    match descriptor.build_mode() {
        TreatmentBuildMode::Compiled(build_fn, desc) => Ok(Arc::new(CompiledBuilder::new(Arc::downgrade(&world), desc, build_fn))),
        TreatmentBuildMode::Source(desc) => Ok(Arc::new(SourceBuilder::new(Arc::downgrade(&world), desc))),
        TreatmentBuildMode::Designed() => {

            let designed_descriptor = Arc::clone(descriptor).downcast_arc::<Treatment>().unwrap();
            Ok(Arc::new(DesignedBuilder::new(Arc::downgrade(&world), designed_descriptor.design()?)))
        }
    }
}

