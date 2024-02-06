//! Module dedicated to AssignedGeneric semantic analysis.

use super::common::Node;
use super::DeclarativeElement;
use super::Type;
use super::TypeFlow;
use crate::error::ScriptError;
use crate::text::Generic as TextGeneric;
use crate::ScriptResult;
use std::sync::{Arc, RwLock, Weak};

/// Structure managing and describing semantic of an assigned generic.
///
/// It owns the whole [text parameter](TextParameter).
#[derive(Debug)]
pub struct AssignedGeneric {
    pub text: TextGeneric,

    pub scope: Weak<RwLock<dyn DeclarativeElement>>,

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
    pub fn new(
        scope: Arc<RwLock<dyn DeclarativeElement>>,
        text: TextGeneric,
    ) -> ScriptResult<Arc<RwLock<Self>>> {
        let result = ScriptResult::new_success(());

        result
            .and_then(|_| {
                if !text.traits.is_empty() {
                    ScriptResult::new_failure(ScriptError::invalid_generic(
                        178,
                        text.r#type.name.clone(),
                    ))
                } else {
                    ScriptResult::new_success(())
                }
            })
            .and_then(|_| Type::new(Arc::clone(&scope), text.r#type.clone()))
            .and_then(|assigned_type| {
                if assigned_type.flow != TypeFlow::Block {
                    ScriptResult::new_failure(ScriptError::flow_forbidden(168, text.r#type.name))
                } else {
                    ScriptResult::new_success(Arc::new(RwLock::new(Self {
                        text,
                        r#type: assigned_type,
                        scope: Arc::downgrade(&scope),
                    })))
                }
            })
    }
}

impl Node for AssignedGeneric {}
