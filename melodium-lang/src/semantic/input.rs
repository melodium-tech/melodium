//! Module dedicated to Input semantic analysis.

use super::common::Node;
use super::r#type::Type;
use super::treatment::Treatment;
use crate::text::Parameter as TextParameter;
use crate::{error::ScriptError, ScriptResult};
use melodium_common::descriptor::Input as InputDescriptor;
use std::sync::{Arc, RwLock, Weak};

/// Structure managing and describing semantic of an input.
///
/// It owns the whole [text parameter](TextParameter).
#[derive(Debug)]
pub struct Input {
    pub text: TextParameter,

    pub treatment: Weak<RwLock<Treatment>>,

    pub name: String,
    pub r#type: Type,
}

impl Input {
    /// Create a new semantic input, based on textual parameter.
    ///
    /// * `treatment`: the parent treatment that owns this input.
    /// * `text`: the textual parameter.
    ///
    /// # Note
    /// Only parent-child relationships are made at this step. Other references can be made afterwards using the [Node trait](Node).
    ///
    pub fn new(
        treatment: Arc<RwLock<Treatment>>,
        text: TextParameter,
    ) -> ScriptResult<Arc<RwLock<Self>>> {
        let mut result = ScriptResult::new_success(());

        let borrowed_treatment = treatment.read().unwrap();

        let input = borrowed_treatment.find_input(&text.name.string);
        if input.is_some() {
            result = result.and_degrade_failure(ScriptResult::new_failure(
                ScriptError::already_declared(129, text.name.clone()),
            ));
        }

        if text.value.is_some() {
            result = result.and_degrade_failure(ScriptResult::new_failure(
                ScriptError::default_forbidden(131, text.name.clone()),
            ));
        }

        if let Some(text_type) = text.r#type.clone() {
            result
                .and_degrade_failure(Type::new(text_type))
                .and_then(|r#type| {
                    ScriptResult::new_success(Arc::<RwLock<Self>>::new(RwLock::new(Self {
                        treatment: Arc::downgrade(&treatment),
                        name: text.name.string.clone(),
                        text,
                        r#type,
                    })))
                })
        } else {
            result.and_degrade_failure(ScriptResult::new_failure(ScriptError::missing_type(
                132,
                text.name.clone(),
            )))
        }
    }

    pub fn make_descriptor(&self) -> ScriptResult<InputDescriptor> {
        self.r#type
            .make_descriptor()
            .and_then(|(described_type, flow)| {
                ScriptResult::new_success(InputDescriptor::new(
                    &self.name,
                    described_type,
                    flow,
                    self.text
                        .annotations
                        .as_ref()
                        .map(|annotations| {
                            annotations
                                .annotations
                                .iter()
                                .filter_map(|annotation| annotation.as_attribute())
                                .collect()
                        })
                        .unwrap_or_default(),
                ))
            })
    }
}

impl Node for Input {}
