#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

#[macro_use]
extern crate lazy_static;

mod building;
pub mod debug;
pub mod descriptor;
pub mod design;
pub mod designer;
pub mod engine;
pub mod error;
pub(crate) mod ids;
mod transmission;
mod world;

pub use engine::Engine;
pub use error::{LogicError, LogicErrors, LogicResult};
use melodium_common::{descriptor::Collection, executive::Level};
use std::sync::Arc;

pub fn new_engine(
    collection: Arc<Collection>,
    log_level: Level,
    debug_level: crate::debug::DebugLevel,
) -> Arc<dyn Engine> {
    world::World::new(collection, log_level, debug_level)
}

pub mod build {
    pub use crate::building::{BuildId, ContextualEnvironment, GenesisEnvironment, HostTreatment};
}

pub use ids::{execution_group_id, execution_run_id};

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");
