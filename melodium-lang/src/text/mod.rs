//! Proceed to basic text parsing and analysis of Mélodium scripts.
//!
//! The main type of this module is [Script](Script), aimed to handle and build a syntax tree of a script.
//! Other types may be useful only for extracting specific contents, and are primarily available as branches of tree built by [Script](Script).
//! All the parsing and extraction heavily relies on the submodule [word](word).
//!

pub mod annotation;
pub mod common;
pub mod connection;
pub mod function;
pub mod instanciation;
pub mod model;
pub mod parameter;
pub mod requirement;
pub mod script;
pub mod treatment;
pub mod r#type;
pub mod r#use;
pub mod value;
pub mod word;

pub use annotation::{Annotation, CommentsAnnotations};
pub use connection::Connection;
pub use function::Function;
pub use instanciation::Instanciation;
pub use model::Model;
pub use parameter::Parameter;
pub use r#type::Type;
pub use r#use::Use;
pub use requirement::Requirement;
pub use script::Script;
pub use treatment::Treatment;
pub use value::Value;
pub use word::Position;
pub use word::PositionnedString;
