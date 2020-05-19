
//! Proceed to basic text parsing and analysis of Mélodium scripts.
//! 
//! The main type of this module is `Script`, aimed to handle and build a syntax tree of a script.
//! Other types may be useful only for extracting specific contents, and are primarily available as branches and leafs of tree built by `Script`.
//!

pub mod script;

pub mod word;

pub mod annotation;
pub mod connection;
pub mod model;
pub mod parameter;
pub mod sequence;
pub mod treatment;
pub mod r#type;
pub mod value;

pub use script::Script;
pub use annotation::Annotation;
pub use connection::Connection;
pub use model::Model;
pub use parameter::Parameter;
pub use sequence::Sequence;
pub use treatment::Treatment;
pub use r#type::Type;
pub use value::Value;
