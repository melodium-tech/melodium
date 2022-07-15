
use crate::core::prelude::*;

pub mod engine;
pub mod engine_end;
pub mod engine_ready;
pub mod stdin;
pub mod stdin_read;
pub mod stdin_close;
pub mod stdout_write;

pub fn register(mut c: &mut CollectionPool) {

    c.models.insert(&(engine::EngineModel::descriptor() as Arc<dyn ModelDescriptor>));
    c.models.insert(&(stdin::StdinModel::descriptor() as Arc<dyn ModelDescriptor>));

    engine_end::engine_end_treatment::register(&mut c);
    engine_ready::engine_ready_treatment::register(&mut c);
    stdin_read::stdin_read_treatment::register(&mut c);
    stdin_close::stdin_close_treatment::register(&mut c);
    stdout_write::stdout_write_treatment::register(&mut c);

}
