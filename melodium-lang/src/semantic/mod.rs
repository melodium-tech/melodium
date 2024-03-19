//! Proceed to semantic analysis and management of Mélodium scripts.
//!
//! The main types of this more are [Tree] and [Script], which are respectively the semantic tree holding structure and the entry point of semantically managed scripts.
//! This module is dependant on the [text module](super::text) for building a semantic tree, and is similarly organized.

mod assignative_element;
mod assigned_generic;
mod assigned_model;
mod assigned_parameter;
mod common;
mod connection;
mod declarative_element;
mod declared_generic;
mod declared_model;
mod declared_parameter;
mod function_call;
mod input;
mod model;
mod model_instanciation;
mod output;
mod requirement;
mod script;
mod treatment;
mod treatment_instanciation;
mod r#type;
mod r#use;
mod value;
mod variability;

pub use assignative_element::{AssignativeElement, AssignativeElementType};
pub use assigned_generic::AssignedGeneric;
pub use assigned_model::AssignedModel;
pub use assigned_parameter::AssignedParameter;
pub use common::{Node, Reference, Tree};
pub use connection::Connection;
pub use declarative_element::{DeclarativeElement, DeclarativeElementType, NoneDeclarativeElement};
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
