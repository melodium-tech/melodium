//! Module dedicated to DeclaredGeneric semantic analysis.

use super::common::Node;
use super::{DeclarativeElement, Type, TypeFlow};
use crate::error::ScriptError;
use crate::text::Generic as TextGeneric;
use crate::ScriptResult;
use melodium_common::descriptor::{Collection, DescribedType};
use std::sync::{Arc, RwLock, Weak};

/// Structure managing and describing semantic of a declared generic.
///
/// It owns the whole [text parameter](TextParameter).
#[derive(Debug)]
pub struct DeclaredGeneric {
    pub text: TextGeneric,

    pub parent: Weak<RwLock<dyn DeclarativeElement>>,

    pub name: String,
    pub traits: Vec<String>,
}

impl DeclaredGeneric {
    /// Create a new semantic declaration of generic, based on textual generic.
    ///
    /// * `parent`: the parent element owning this declaterion.
    /// * `text`: the textual generic.
    ///
    /// # Note
    /// Only parent-child relationships are made at this step. Other references can be made afterwards using the [Node trait](Node).
    ///
    pub fn new(
        parent: Arc<RwLock<dyn DeclarativeElement>>,
        text: TextGeneric,
    ) -> ScriptResult<Arc<RwLock<Self>>> {
        let result = ScriptResult::new_success(());

        result
            .and_then(|_| Type::new(Arc::clone(&parent), text.r#type.clone()))
            .and_then(|declared_type| {
                if declared_type.flow != TypeFlow::Block {
                    ScriptResult::new_failure(ScriptError::flow_forbidden(174, text.r#type.name))
                } else {
                    declared_type.make_descriptor(&Collection::new()).and_then(
                        |(described_type, _)| match described_type {
                            DescribedType::Generic(generic) => {
                                ScriptResult::new_success(Arc::new(RwLock::new(Self {
                                    name: generic.name,
                                    parent: Arc::downgrade(&parent),
                                    traits: text
                                        .traits
                                        .iter()
                                        .map(|ps| ps.string.clone())
                                        .collect(),
                                    text,
                                })))
                            }
                            _ => ScriptResult::new_failure(ScriptError::invalid_generic(
                                175,
                                text.r#type.name,
                            )),
                        },
                    )
                }
            })
    }
}

impl Node for DeclaredGeneric {}
