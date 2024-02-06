//! Proceed to semantic analysis and management of Mélodium scripts.
//!
//! The main types of this more are [Tree] and [Script], which are respectively the semantic tree holding structure and the entry point of semantically managed scripts.
//! This module is dependant on the [text module](super::text) for building a semantic tree, and is similarly organized.

pub mod assignative_element;
pub mod assigned_generic;
pub mod assigned_model;
pub mod assigned_parameter;
pub mod common;
pub mod connection;
pub mod declarative_element;
pub mod declared_generic;
pub mod declared_model;
pub mod declared_parameter;
pub mod function_call;
pub mod input;
pub mod model;
pub mod model_instanciation;
pub mod output;
pub mod requirement;
pub mod script;
pub mod treatment;
pub mod treatment_instanciation;
pub mod r#type;
pub mod r#use;
pub mod value;
pub mod variability;

pub use assignative_element::{AssignativeElement, AssignativeElementType};
pub use assigned_generic::AssignedGeneric;
pub use assigned_model::AssignedModel;
pub use assigned_parameter::AssignedParameter;
pub use common::{Node, Reference, Tree};
pub use connection::Connection;
pub use declarative_element::{DeclarativeElement, DeclarativeElementType};
pub use declared_generic::DeclaredGeneric;
pub use declared_model::DeclaredModel;
pub use declared_parameter::DeclaredParameter;
pub use function_call::FunctionCall;
pub use input::Input;
pub use model::Model;
pub use model_instanciation::ModelInstanciation;
pub use output::Output;
pub use r#type::{Type, TypeFlow};
pub use r#use::Use;
pub use requirement::Requirement;
pub use script::Script;
pub use treatment::Treatment;
pub use treatment_instanciation::TreatmentInstanciation;
pub use value::{Value, ValueContent};
pub use variability::Variability;
