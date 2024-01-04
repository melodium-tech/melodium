//! Module dedicated to AssignedParameter semantic analysis.

use super::assignative_element::AssignativeElement;
use super::common::Node;
use super::value::Value;
use crate::error::ScriptError;
use crate::text::Parameter as TextParameter;
use crate::ScriptResult;
use melodium_engine::{designer::Parameter as ParameterDesigner, LogicResult};
use std::sync::{Arc, RwLock, Weak};

/// Structure managing and describing semantic of an assigned parameter.
///
/// A _assigned_ parameter is a parameter for which name and value are expected, but _no_ type.
/// It is used by [Treatments](super::Treatment) and [Models](super::Model).
///
/// It owns the whole [text parameter](../../text/parameter/struct.Parameter.html).
#[derive(Debug)]
pub struct AssignedParameter {
    pub text: TextParameter,

    pub parent: Weak<RwLock<dyn AssignativeElement>>,

    pub name: String,
    pub value: Arc<RwLock<Value>>,
}

impl AssignedParameter {
    /// Create a new semantic assigned parameter, based on textual parameter.
    ///
    /// * `parent`: the parent owning this parameter.
    /// * `text`: the textual parameter.
    ///
    /// # Note
    /// Only parent-child relationships are made at this step. Other references can be made afterwards using the [Node trait](Node).
    ///
    pub fn new(
        parent: Arc<RwLock<dyn AssignativeElement>>,
        text: TextParameter,
    ) -> ScriptResult<Arc<RwLock<Self>>> {
        let mut result = ScriptResult::new_success(());

        let borrowed_parent = parent.read().unwrap();

        let parameter = borrowed_parent.find_assigned_parameter(&text.name.string);
        if parameter.is_some() {
            result = result.and_degrade_failure(ScriptResult::new_failure(
                ScriptError::already_assigned(146, text.name.clone()),
            ));
        }

        if let Some(_erroneous_type) = &text.r#type {
            result = result.and_degrade_failure(ScriptResult::new_failure(
                ScriptError::type_forbidden(150, text.name.clone()),
            ));
        }

        if let Some(text_value) = text.value.clone() {
            result
                .and_then(|_| {
                    Value::new(
                        borrowed_parent.associated_declarative_element(),
                        text_value.clone(),
                    )
                })
                .and_then(|value| {
                    ScriptResult::new_success(Arc::<RwLock<Self>>::new(RwLock::new(Self {
                        name: text.name.string.clone(),
                        text,
                        parent: Arc::downgrade(&parent),
                        value,
                    })))
                })
        } else {
            result.and_degrade_failure(ScriptResult::new_failure(ScriptError::missing_value(
                147,
                text.name.clone(),
            )))
        }
    }

    pub fn make_design(&self, designer: &Arc<RwLock<ParameterDesigner>>) -> ScriptResult<()> {
        let mut designer = designer.write().unwrap();

        let described_type_result = designer.described_type();
        if let Some(Some(described_type)) = described_type_result.success() {
            let designed_value = self
                .value
                .read()
                .unwrap()
                .make_designed_value(&designer, &described_type);
            designed_value
                .and_then(|designed_value| ScriptResult::from(designer.set_value(designed_value)))
        } else {
            ScriptResult::from(described_type_result.and(LogicResult::new_success(())))
        }
    }
}

impl Node for AssignedParameter {
    fn children(&self) -> Vec<Arc<RwLock<dyn Node>>> {
        let mut children: Vec<Arc<RwLock<dyn Node>>> = Vec::new();

        children.push(Arc::clone(&self.value) as Arc<RwLock<dyn Node>>);

        children
    }
}
