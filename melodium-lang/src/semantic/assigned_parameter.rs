//! Module dedicated to AssignedParameter semantic analysis.

use super::assignative_element::AssignativeElement;
use super::common::Node;
use super::value::Value;
use crate::error::{wrap_logic_error, ScriptError};
use crate::text::Parameter as TextParameter;
use melodium_engine::designer::Parameter as ParameterDesigner;
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
    ) -> Result<Arc<RwLock<Self>>, ScriptError> {
        let value;
        {
            let borrowed_parent = parent.read().unwrap();

            let parameter = borrowed_parent.find_assigned_parameter(&text.name.string);
            if parameter.is_some() {
                return Err(ScriptError::semantic(
                    "Parameter '".to_string() + &text.name.string + "' is already assigned.",
                    text.name.position,
                ));
            }

            if text.value.is_some() {
                value = Value::new(
                    borrowed_parent.associated_declarative_element(),
                    text.value.as_ref().unwrap().clone(),
                )?;
            } else {
                return Err(ScriptError::semantic(
                    "Parameter '".to_string() + &text.name.string + "' is missing value.",
                    text.name.position,
                ));
            }
        }

        Ok(Arc::<RwLock<Self>>::new(RwLock::new(Self {
            name: text.name.string.clone(),
            text,
            parent: Arc::downgrade(&parent),
            value,
        })))
    }

    pub fn make_design(
        &self,
        designer: &Arc<RwLock<ParameterDesigner>>,
    ) -> Result<(), ScriptError> {
        let mut designer = designer.write().unwrap();
        let descriptor = designer
            .parent_descriptor()
            .upgrade()
            .unwrap()
            .parameters()
            .get(&self.name)
            .unwrap()
            .clone();

        let value = self
            .value
            .read()
            .unwrap()
            .make_designed_value(&designer, descriptor.datatype())?;

        wrap_logic_error!(designer.set_value(value), self.text.name.position);

        Ok(())
    }
}

impl Node for AssignedParameter {
    fn children(&self) -> Vec<Arc<RwLock<dyn Node>>> {
        let mut children: Vec<Arc<RwLock<dyn Node>>> = Vec::new();

        children.push(Arc::clone(&self.value) as Arc<RwLock<dyn Node>>);

        children
    }
}
