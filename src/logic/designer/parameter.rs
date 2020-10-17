
use std::rc::{Rc, Weak};
use std::cell::RefCell;
use super::super::error::LogicError;
use super::super::ParameterizedDescriptor;
use super::sequence::Sequence;
use super::value::Value;

pub struct Parameter {

    sequence: Weak<RefCell<Sequence>>,
    descriptor: Rc<dyn ParameterizedDescriptor>,
    name: String,
    value: Option<Value>,
}

impl Parameter {
    pub fn new(sequence: &Rc<RefCell<Sequence>>, descriptor: &Rc<dyn ParameterizedDescriptor>, name: &str) -> Self {
        Self {
            sequence: Rc::downgrade(sequence),
            descriptor: Rc::clone(descriptor),
            name: name.to_string(),
            value: None,
        }
    }

    pub fn descriptor(&self) -> &Rc<dyn ParameterizedDescriptor> {
        &self.descriptor
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn validate(&self) -> Result<(), LogicError> {
        Ok(())
    }
}
