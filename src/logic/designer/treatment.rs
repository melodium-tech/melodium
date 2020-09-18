
use std::rc::Rc;
use super::super::TreatmentDescriptor;

pub struct Treatment {

    descriptor: Rc<dyn TreatmentDescriptor>,
    name: String,

}

impl Treatment {
    pub fn new(descriptor: &Rc<dyn TreatmentDescriptor>, name: &str) -> Self {
        Self {
            descriptor: Rc::clone(descriptor),
            name: name.to_string(),
        }
    }

    pub fn descriptor(&self) -> &Rc<dyn TreatmentDescriptor> {
        &self.descriptor
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
