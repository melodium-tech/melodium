
use super::common::Node;

use std::rc::{Rc, Weak};
use std::cell::RefCell;
use crate::script::error::ScriptError;
use crate::script::text::Parameter as TextParameter;
use crate::script::text::Value as TextValue;

use super::assignative_element::AssignativeElement;
use super::declarative_element::DeclarativeElementType;
use super::common::Reference;
use super::declared_model::DeclaredModel;

pub struct AssignedModel {
    pub text: TextParameter,

    pub parent: Weak<RefCell<dyn AssignativeElement>>,

    pub name: String,
    pub model: Reference<DeclaredModel>,
}

impl AssignedModel {
    pub fn new(parent: Rc<RefCell<dyn AssignativeElement>>, text: TextParameter) -> Result<Rc<RefCell<Self>>, ScriptError> {

        let referred_model_name;
        {
            let borrowed_parent = parent.borrow();

            let assigned_model = borrowed_parent.find_assigned_model(&text.name.string);
            if assigned_model.is_some() {
                return Err(ScriptError::semantic("Model '".to_string() + &text.name.string + "' is already assigned.", text.name.position))
            }

            if let Some(erroneous_type) = &text.r#type {
                return Err(ScriptError::semantic("Model assignation cannot be typed.".to_string(), erroneous_type.name.position))
            }

            if let Some(TextValue::Name(model_name)) = &text.value {
                referred_model_name = model_name.string.clone();
            }
            else {
                return Err(ScriptError::semantic("Model assignation require a name.".to_string(), text.name.position))
            }
        }

        Ok(Rc::<RefCell<Self>>::new(RefCell::new(Self {
            name: text.name.string.clone(),
            text,
            parent: Rc::downgrade(&parent),
            model: Reference {
                name: referred_model_name,
                reference: None,
            },
        })))
    }
}

impl Node for AssignedModel {
    fn make_references(&mut self) -> Result<(), ScriptError> {

        if self.model.reference.is_none() {
            
            let rc_parent = self.parent.upgrade().unwrap();
            let borrowed_parent = rc_parent.borrow();

            let rc_declarative_element = borrowed_parent.associated_declarative_element();
            let borrowed_declarative_element = rc_declarative_element.borrow();
            let refered_model = match &borrowed_declarative_element.declarative_element() {
                DeclarativeElementType::Sequence(s) => s.find_declared_model(&self.model.name),
                _ => None,
            };

            if let Some(rc_refered_model) = refered_model {
                self.model.reference = Some(Rc::downgrade(rc_refered_model));
            }
            else {
                return Err(ScriptError::semantic("Unkown name '".to_string() + &self.name + "' in declared models.", self.text.name.position));
            }
        }

        Ok(())
    }
}
