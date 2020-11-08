
use std::rc::Rc;
use super::super::error::LogicError;
use super::super::ParameterizedDescriptor;
use super::value::Value;

pub struct Parameter {

    descriptor: Rc<dyn ParameterizedDescriptor>,
    name: String,
    value: Option<Value>,
}

impl Parameter {
    pub fn new(descriptor: &Rc<dyn ParameterizedDescriptor>, name: &str) -> Self {
        Self {
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

    pub fn set_value(&mut self, value: Value) -> Result<(), LogicError> {
        
        Ok(())
    }

    pub fn validate(&self) -> Result<(), LogicError> {
        Ok(())
    }
}
