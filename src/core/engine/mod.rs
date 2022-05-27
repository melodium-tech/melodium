
use crate::core::prelude::*;

pub mod engine;
pub mod engine_ready;

pub fn register(mut c: &mut CollectionPool) {

    c.models.insert(&(engine::EngineModel::descriptor() as Arc<dyn ModelDescriptor>));

    engine_ready::engine_ready_treatment::register(&mut c);

}
