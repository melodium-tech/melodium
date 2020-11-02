
use std::rc::{Rc, Weak};
use std::cell::RefCell;
use std::collections::HashMap;
use super::super::error::LogicError;
use super::super::ModelDescriptor;
use super::sequence::Sequence;
use super::parameter::Parameter;

pub struct ModelInstanciation {

    sequence: Weak<RefCell<Sequence>>,
    descriptor: Rc<dyn ModelDescriptor>,
    name: String,
    parameters: HashMap<String, Rc<RefCell<Parameter>>>,
}

impl ModelInstanciation {
    pub fn new(sequence: &Rc<RefCell<Sequence>>, descriptor: &Rc<dyn ModelDescriptor>, name: &str) -> Self {
        Self {
            sequence: Rc::downgrade(sequence),
            descriptor: Rc::clone(descriptor),
            name: name.to_string(),
            parameters: HashMap::with_capacity(descriptor.parameters().len()),
        }
    }

    pub fn descriptor(&self) -> &Rc<dyn ModelDescriptor> {
        &self.descriptor
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn validate(&self) -> Result<(), LogicError> {
        Ok(())
    }

    pub fn register(&self) -> Result<(), LogicError> {
        
        self.validate()?;

        todo!();
    }
}