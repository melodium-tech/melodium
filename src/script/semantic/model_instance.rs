
use super::common::Node;

use std::rc::Rc;
use std::cell::RefCell;
use crate::script::error::ScriptError;
use crate::script::text::Parameter as TextParameter;

use super::common::Reference;
use super::sequence::Sequence;
use super::r#use::Use;
use super::model_instanciation::ModelInstanciation;

pub struct ModelInstance {
    pub text: TextParameter,

    pub sequence: Rc<RefCell<Sequence>>,

    pub name: String,
    pub refers: RefersTo,
}

pub enum RefersTo {
    Unkown(Reference<()>),
    Use(Reference<Use>),
    Sequence(Reference<ModelInstanciation>),
}

impl ModelInstance {
    pub fn new(sequence: Rc<RefCell<Sequence>>, text: TextParameter) -> Result<Rc<RefCell<Self>>, ScriptError> {

        let refers_string;
        {
            let borrowed_sequence = sequence.borrow();

            let model_instance = borrowed_sequence.find_model_instance(&text.name.string);
            if model_instance.is_some() {
                return Err(ScriptError::semantic("Model '".to_string() + &text.name.string + "' is already declared.", text.name.position))
            }
            

            if let Some(r#type) = &text.r#type {

                if r#type.structure.is_some() {
                    return Err(ScriptError::semantic("Model '".to_string() + &text.name.string + "' cannot have type structure.", text.name.position))
                }

                refers_string = r#type.name.clone();
            }
            else {
                return Err(ScriptError::semantic("Model '".to_string() + &text.name.string + "' do not have type.", text.name.position))
            }

            if text.value.is_some() {
                return Err(ScriptError::semantic("Model '".to_string() + &text.name.string + "' cannot be assigned to a value.", text.name.position))
            }
        }

        Ok(Rc::<RefCell<Self>>::new(RefCell::new(Self {
            sequence,
            name: text.name.string.clone(),
            text,
            refers: RefersTo::Unkown(Reference::new(refers_string.string))
        })))
    }
}

impl Node for ModelInstance {
    fn make_references(&mut self) -> Result<(), ScriptError> {
        todo!();
    }
}
