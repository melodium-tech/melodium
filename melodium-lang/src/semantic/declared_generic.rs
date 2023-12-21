//! Module dedicated to DeclaredGeneric semantic analysis.

use super::common::Node;
use super::{Type, TypeContent, TypeFlow};
use crate::error::ScriptError;
use crate::text::Generic as TextGeneric;
use crate::ScriptResult;
use std::sync::{Arc, RwLock};

/// Structure managing and describing semantic of a declared generic.
///
/// It owns the whole [text parameter](TextParameter).
#[derive(Debug)]
pub struct DeclaredGeneric {
    pub text: TextGeneric,
    pub name: String,
}

impl DeclaredGeneric {
    /// Create a new semantic declaretion of generic, based on textual generic.
    ///
    /// * `parent`: the parent element owning this declaterion.
    /// * `text`: the textual generic.
    ///
    /// # Note
    /// Only parent-child relationships are made at this step. Other references can be made afterwards using the [Node trait](Node).
    ///
    pub fn new(text: TextGeneric) -> ScriptResult<Arc<RwLock<Self>>> {
        let result = ScriptResult::new_success(());

        result
            .and_then(|_| Type::new(text.r#type.clone()))
            .and_then(|declared_type| {
                if declared_type.flow != TypeFlow::Block {
                    ScriptResult::new_failure(ScriptError::flow_forbidden(174, text.r#type.name))
                } else {
                    match declared_type.content {
                        TypeContent::Other(name) => {
                            ScriptResult::new_success(Arc::new(RwLock::new(Self { text, name })))
                        }
                        _ => ScriptResult::new_failure(ScriptError::invalid_generic(
                            175,
                            text.r#type.name,
                        )),
                    }
                }
            })
    }
}

impl Node for DeclaredGeneric {}
