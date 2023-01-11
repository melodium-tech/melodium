
//! Proceed to basic text parsing and analysis of Mélodium scripts.
//! 
//! The main type of this module is [Script](./script/struct.Script.html), aimed to handle and build a syntax tree of a script.
//! Other types may be useful only for extracting specific contents, and are primarily available as branches of tree built by [Script](./script/struct.Script.html).
//! All the parsing and extraction heavily relies on the submodule [word](./word/index.html).
//!

pub mod script;

pub mod word;
pub mod common;

pub mod annotation;
pub mod connection;
pub mod function;
pub mod instanciation;
pub mod model;
pub mod parameter;
pub mod requirement;
pub mod sequence;
pub mod r#type;
pub mod r#use;
pub mod value;

pub use script::Script;
pub use annotation::Annotation;
pub use connection::Connection;
pub use function::Function;
pub use instanciation::Instanciation;
pub use model::Model;
pub use parameter::Parameter;
pub use word::Position;
pub use word::PositionnedString;
pub use requirement::Requirement;
pub use sequence::Sequence;
pub use r#type::Type;
pub use r#use::Use;
pub use value::Value;
