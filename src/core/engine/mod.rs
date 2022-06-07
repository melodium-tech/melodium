
use crate::core::prelude::*;

#[cfg(not(target_family = "windows"))]
pub mod engine;
#[cfg(target_family = "windows")]
pub mod engine_windows;
#[cfg(target_family = "windows")]
pub use engine_windows as engine;

pub mod engine_end;
pub mod engine_ready;
pub mod engine_read;
pub mod engine_signals;
pub mod engine_write;

pub fn register(mut c: &mut CollectionPool) {

    c.models.insert(&(engine::EngineModel::descriptor() as Arc<dyn ModelDescriptor>));

    engine_end::engine_end_treatment::register(&mut c);
    engine_ready::engine_ready_treatment::register(&mut c);
    engine_read::engine_read_treatment::register(&mut c);
    engine_signals::engine_sighup_treatment::register(&mut c);
    engine_signals::engine_sigterm_treatment::register(&mut c);
    engine_write::engine_write_treatment::register(&mut c);

}
