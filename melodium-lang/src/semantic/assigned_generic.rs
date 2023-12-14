//! Module dedicated to AssignedGeneric semantic analysis.

use super::common::Node;
use super::Type;
use super::TypeFlow;
use crate::error::ScriptError;
use crate::text::Generic as TextGeneric;
use crate::ScriptResult;
use std::sync::{Arc, RwLock};

/// Structure managing and describing semantic of an assigned generic.
///
/// It owns the whole [text parameter](TextParameter).
#[derive(Debug)]
pub struct AssignedGeneric {
    pub text: TextGeneric,
    pub r#type: Type,
}

impl AssignedGeneric {
    /// Create a new semantic assignation of generic, based on textual generic.
    ///
    /// * `parent`: the parent element owning this assignation.
    /// * `text`: the textual generic.
    ///
    /// # Note
    /// Only parent-child relationships are made at this step. Other references can be made afterwards using the [Node trait](Node).
    ///
    pub fn new(text: TextGeneric) -> ScriptResult<Arc<RwLock<Self>>> {
        let result = ScriptResult::new_success(());

        result
            .and_then(|_| Type::new(text.r#type.clone()))
            .and_then(|assigned_type| {
                if assigned_type.flow != TypeFlow::Block {
                    ScriptResult::new_failure(ScriptError::flow_forbidden(168, text.r#type.name))
                } else {
                    ScriptResult::new_success(Arc::new(RwLock::new(Self {
                        text,
                        r#type: assigned_type,
                    })))
                }
            })
    }
}

impl Node for AssignedGeneric {}
