
use std::rc::Rc;
use super::super::ModelDescriptor;

pub struct ModelInstanciation {

    descriptor: Rc<dyn ModelDescriptor>,
    name: String,
}

impl ModelInstanciation {
    pub fn new(descriptor: &Rc<dyn ModelDescriptor>, name: &str) -> Self {
        Self {
            descriptor: Rc::clone(descriptor),
            name: name.to_string(),
        }
    }

    pub fn descriptor(&self) -> &Rc<dyn ModelDescriptor> {
        &self.descriptor
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}