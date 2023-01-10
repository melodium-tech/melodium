#[macro_use]
extern crate lazy_static;

pub mod building;
pub mod descriptor;
pub mod design;
pub mod designer;
pub mod engine;
pub mod error;
pub mod executive;
pub mod transmission;
pub mod world;

pub use engine::Engine;
use melodium_common::descriptor::Collection;
use std::sync::Arc;

pub fn new_engine(collection: Arc<Collection>) -> Arc<dyn Engine> {
    world::World::new(collection)
}
