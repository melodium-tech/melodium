//!
//! Mélodium core engine implementation.
//!
//! This crate provides the core Mélodium engine.
//! The [descriptor](crate::descriptor) module provides descriptors allowing design to be made.
//! Everything needed to design [models](crate::designer::Model) and [treatments](crate::designer::Treatment) is provided in the [designer](crate::designer) module.
//! The [design](crate::design) module provides purely descriptive design without mutable interaction.
//!
//! The [engine](crate::Engine) trait provides interactions with a core Mélodium engine, that can be instancied through [new_engine](crate::new_engine) function.
//!

#[macro_use]
extern crate lazy_static;

mod building;
pub mod descriptor;
pub mod design;
pub mod designer;
pub mod engine;
pub mod error;
mod executive;
mod transmission;
mod world;

pub use engine::Engine;
pub use error::LogicError;
use melodium_common::descriptor::Collection;
use std::sync::Arc;

pub fn new_engine(collection: Arc<Collection>) -> Arc<dyn Engine> {
    world::World::new(collection)
}
