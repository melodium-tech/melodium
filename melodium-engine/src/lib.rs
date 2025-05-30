#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

#[macro_use]
extern crate lazy_static;

mod building;
pub mod descriptor;
pub mod design;
pub mod designer;
pub mod engine;
pub mod error;
mod transmission;
mod world;

pub use engine::Engine;
pub use error::{LogicError, LogicErrors, LogicResult};
use melodium_common::descriptor::Collection;
use std::sync::Arc;

pub fn new_engine(collection: Arc<Collection>) -> Arc<dyn Engine> {
    world::World::new(collection)
}
