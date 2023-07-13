#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

#[macro_use]
extern crate lazy_static;

pub mod error;
pub mod path;
#[cfg_attr(docsrs, doc(cfg(feature = "restitution")))]
#[cfg(feature = "restitution")]
pub mod restitution;
pub mod semantic;
pub mod text;

pub use error::{ScriptError, ScriptResult};
pub use path::Path;
