//! Module dedicated to AssignedModel semantic analysis.

use super::assignative_element::AssignativeElement;
use super::common::Node;
use super::common::Reference;
use super::declarative_element::DeclarativeElementType;
use super::declared_model::DeclaredModel;
use crate::error::ScriptError;
use crate::path::Path;
use crate::text::Parameter as TextParameter;
use crate::text::Value as TextValue;
use std::sync::{Arc, RwLock, Weak};

/// Structure managing and describing semantic of an assigned model.
///
/// It owns the whole [text parameter](TextParameter).
#[derive(Debug)]
pub struct AssignedModel {
    pub text: TextParameter,

    pub parent: Weak<RwLock<dyn AssignativeElement>>,

    pub name: String,
    pub model: Reference<DeclaredModel>,
}

impl AssignedModel {
    /// Create a new semantic assignation of model, based on textual parameter.
    ///
    /// * `parent`: the parent element owning this assignation.
    /// * `text`: the textual model.
    ///
    /// # Note
    /// Only parent-child relationships are made at this step. Other references can be made afterwards using the [Node trait](Node).
    ///
    pub fn new(
        parent: Arc<RwLock<dyn AssignativeElement>>,
        text: TextParameter,
    ) -> Result<Arc<RwLock<Self>>, ScriptError> {
        let referred_model_name;
        {
            let borrowed_parent = parent.read().unwrap();

            let assigned_model = borrowed_parent.find_assigned_model(&text.name.string);
            if assigned_model.is_some() {
                return Err(ScriptError::semantic(
                    "Model '".to_string() + &text.name.string + "' is already assigned.",
                    text.name.position,
                ));
            }

            if let Some(erroneous_type) = &text.r#type {
                return Err(ScriptError::semantic(
                    "Model assignation cannot be typed.".to_string(),
                    erroneous_type.name.position,
                ));
            }

            if let Some(TextValue::Name(model_name)) = &text.value {
                referred_model_name = model_name.string.clone();
            } else {
                return Err(ScriptError::semantic(
                    "Model assignation require a name.".to_string(),
                    text.name.position,
                ));
            }
        }

        Ok(Arc::<RwLock<Self>>::new(RwLock::new(Self {
            name: text.name.string.clone(),
            text,
            parent: Arc::downgrade(&parent),
            model: Reference {
                name: referred_model_name,
                reference: None,
            },
        })))
    }
}

impl Node for AssignedModel {
    fn make_references(&mut self, _path: &Path) -> Result<(), ScriptError> {
        if self.model.reference.is_none() {
            let rc_parent = self.parent.upgrade().unwrap();
            let borrowed_parent = rc_parent.read().unwrap();

            let rc_declarative_element = borrowed_parent.associated_declarative_element();
            let borrowed_declarative_element = rc_declarative_element.read().unwrap();
            let refered_model = match &borrowed_declarative_element.declarative_element() {
                DeclarativeElementType::Treatment(t) => t.find_declared_model(&self.model.name),
                _ => None,
            };

            if let Some(rc_refered_model) = refered_model {
                self.model.reference = Some(Arc::downgrade(rc_refered_model));
            } else {
                return Err(ScriptError::semantic(
                    "Unknown name '".to_string() + &self.name + "' in declared models.",
                    self.text.name.position,
                ));
            }
        }

        Ok(())
    }
}
