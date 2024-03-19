//! Proceed to basic text parsing and analysis of Mélodium scripts.
//!
//! The main type of this module is [Script](Script), aimed to handle and build a syntax tree of a script.
//! Other types may be useful only for extracting specific contents, and are primarily available as branches of tree built by [Script](Script).
//! All the parsing and extraction heavily relies on the submodule [word](word).
//!

mod annotation;
mod common;
mod connection;
mod function;
mod generic;
mod instanciation;
mod model;
mod parameter;
mod requirement;
mod script;
mod treatment;
mod r#type;
mod r#use;
mod value;
mod word;

pub use annotation::{Annotation, CommentsAnnotations};
pub use connection::Connection;
pub use function::Function;
pub use generic::Generic;
pub use instanciation::Instanciation;
pub use model::Model;
pub use parameter::Parameter;
pub use r#type::Type;
pub use r#use::Use;
pub use requirement::Requirement;
pub use script::Script;
pub use treatment::Treatment;
pub use value::Value;
pub use word::*;
