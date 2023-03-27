
//! 
//! Mélodium loading engine and utilities.
//! 
//! This crate provides loading logic and processing for the Mélodium environment.
//! 
//! Look at the [Mélodium crate](https://docs.rs/melodium/latest/melodium/)
//! or the [Mélodium Project](https://melodium.tech/) for more detailed information.
//! 

mod content;
mod loader;
mod loading_config;
mod package;

pub use loader::Loader;
pub use loading_config::LoadingConfig;
