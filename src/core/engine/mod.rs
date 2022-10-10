
use crate::core::prelude::*;

pub mod engine;
pub mod stdin;
pub mod stdout_write;

pub fn register(mut c: &mut CollectionPool) {

    c.models.insert(&(engine::EngineModel::descriptor() as std::sync::Arc<dyn ModelDescriptor>));
    engine::engine_ready_source::register(&mut c);
    engine::engine_end_treatment::register(&mut c);

    c.models.insert(&(stdin::StdinModel::descriptor() as std::sync::Arc<dyn ModelDescriptor>));
    stdin::stdin_read_source::register(&mut c);
    stdin::stdin_close_treatment::register(&mut c);
    
    stdout_write::stdout_write_treatment::register(&mut c);

}
