
use std::rc::{Rc, Weak};
use std::cell::RefCell;
use std::collections::HashMap;
use super::super::error::LogicError;
use super::super::TreatmentDescriptor;
use super::sequence::Sequence;
use super::parameter::Parameter;

pub struct Treatment {

    sequence: Weak<RefCell<Sequence>>,
    descriptor: Rc<dyn TreatmentDescriptor>,
    name: String,
    parameters: HashMap<String, Rc<RefCell<Parameter>>>,
}

impl Treatment {
    pub fn new(sequence: &Rc<RefCell<Sequence>>, descriptor: &Rc<dyn TreatmentDescriptor>, name: &str) -> Self {
        Self {
            sequence: Rc::downgrade(sequence),
            descriptor: Rc::clone(descriptor),
            name: name.to_string(),
            parameters: HashMap::with_capacity(descriptor.parameters().len()),
        }
    }

    pub fn descriptor(&self) -> &Rc<dyn TreatmentDescriptor> {
        &self.descriptor
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn add_parameter(&mut self, name: &str) -> Result<Rc<RefCell<Parameter>>, LogicError> {
        todo!();
    }

    pub fn validate(&self) -> Result<(), LogicError> {
        Ok(())
    }
}
