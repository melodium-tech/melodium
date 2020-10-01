
use super::common::Node;

use std::rc::{Rc, Weak};
use std::cell::RefCell;
use crate::script::error::ScriptError;
use crate::script::text::Parameter as TextParameter;
use crate::script::text::word::PositionnedString;

use super::common::Reference;
use super::sequence::Sequence;
use super::r#use::Use;
use super::instancied_model::InstanciedModel;

pub struct DeclaredModel {
    pub text: Option<TextParameter>,

    pub sequence: Weak<RefCell<Sequence>>,

    pub name: String,
    pub refers: RefersTo,
}

pub enum RefersTo {
    Unkown(Reference<()>),
    Use(Reference<Use>),
    InstanciedModel(Reference<InstanciedModel>),
}

impl DeclaredModel {
    pub fn from_instancied_model(instancied_model: Rc<RefCell<InstanciedModel>>) -> Result<Rc<RefCell<Self>>, ScriptError> {
        
        let borrowed_instancied_model = instancied_model.borrow();

        let sequence = borrowed_instancied_model.sequence.upgrade().unwrap();
        let name = borrowed_instancied_model.name.clone();

        let declared_model = Self::make(sequence, borrowed_instancied_model.text.name.clone())?;

        declared_model.borrow_mut().refers = RefersTo::InstanciedModel(Reference {
            name: name,
            reference: Some(Rc::downgrade(&instancied_model))
        });

        Ok(declared_model)
    }

    pub fn new(sequence: Rc<RefCell<Sequence>>, text: TextParameter) -> Result<Rc<RefCell<Self>>, ScriptError> {

        let refers_string;
        if let Some(r#type) = &text.r#type {

            if r#type.structure.is_some() {
                return Err(ScriptError::semantic("Model '".to_string() + &text.name.string + "' cannot have type structure.", text.name.position))
            }

            refers_string = r#type.name.string.clone();
        }
        else {
            return Err(ScriptError::semantic("Model '".to_string() + &text.name.string + "' do not have type.", text.name.position))
        }

        if text.value.is_some() {
            return Err(ScriptError::semantic("Model '".to_string() + &text.name.string + "' cannot be assigned to a value.", text.name.position))
        }

        let declared_model = Self::make(sequence, text.name.clone())?;

        declared_model.borrow_mut().text = Some(text);
        declared_model.borrow_mut().refers = RefersTo::Unkown(Reference::new(refers_string));

        Ok(declared_model)
    }

    fn make(sequence: Rc<RefCell<Sequence>>, name: PositionnedString) -> Result<Rc<RefCell<Self>>, ScriptError> {

        let borrowed_sequence = sequence.borrow();

        let declared_model = borrowed_sequence.find_declared_model(&name.string.clone());
        if declared_model.is_some() {
            return Err(ScriptError::semantic("Model '".to_string() + &name.string.clone() + "' is already declared.", name.position.clone()))
        }

        Ok(Rc::<RefCell<Self>>::new(RefCell::new(Self {
            sequence: Rc::downgrade(&sequence),
            name: name.string.clone(),
            text: None,
            refers: RefersTo::Unkown(Reference::new(name.string))
        })))
    }
}

impl Node for DeclaredModel {
    fn make_references(&mut self) -> Result<(), ScriptError> {
        
        // Reference to an instancied model already been done through Self::from_instancied_model
        // so we only look for reference to a use.
        if let RefersTo::Unkown(reference) = &self.refers {

            let rc_sequence = self.sequence.upgrade().unwrap();
            let borrowed_sequence = rc_sequence.borrow();
            let rc_script = borrowed_sequence.script.upgrade().unwrap();
            let borrowed_script = rc_script.borrow();

            let r#use = borrowed_script.find_use(&reference.name);
            if r#use.is_some() {

                self.refers = RefersTo::Use(Reference{
                    name: reference.name.clone(),
                    reference: Some(Rc::downgrade(r#use.unwrap()))
                });
            }
            else {
                return Err(ScriptError::semantic("'".to_string() + &reference.name + "' is unkown.", self.text.as_ref().unwrap().r#type.as_ref().unwrap().name.position))
            }
        }

        Ok(())
    }
}
