
//! Proceed to semantic analysis and management of Mélodium scripts.
//! 
//! The main types of this more are [Tree](./common/struct.Tree.html) and [Script](./script/struct.Script.html), which are respectively the semantic tree holding structure and the entry point of semantically managed scripts.
//! This module is dependant on the [text module](../text/index.html) for building a semantic tree, and is similarly organized.

pub mod assignative_element;
pub mod assigned_parameter;
pub mod assigned_model;
pub mod common;
pub mod connection;
pub mod declarative_element;
pub mod declared_parameter;
pub mod declared_model;
pub mod function_call;
pub mod input;
pub mod model_instanciation;
pub mod treatment_instanciation;
pub mod model;
pub mod output;
pub mod requirement;
pub mod script;
pub mod treatment;
pub mod r#type;
pub mod r#use;
pub mod value;
pub mod variability;
