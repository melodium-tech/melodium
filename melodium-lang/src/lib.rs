//!
//! Mélodium language implementation.
//!
//! This crate provides language parsing, semantic building, and executive design for the Mélodium language.
//!
//! Look at the [Mélodium crate](https://docs.rs/melodium/latest/melodium/)
//! or the [Mélodium Project](https://melodium.tech/) for more detailed information.
//!

#[macro_use]
extern crate lazy_static;

pub mod error;
pub mod path;
#[cfg(feature = "restitution")]
pub mod restitution;
pub mod semantic;
pub mod text;

pub use error::{ScriptError, ScriptResult};
pub use path::Path;
