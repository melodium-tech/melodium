
use std::rc::Rc;
use std::cell::RefCell;
use super::super::error::LogicError;
use super::super::ConnectionDescriptor;
use super::treatment::Treatment;

pub struct Connection {

    descriptor: Rc<ConnectionDescriptor>,

    output_treatment: Option<Rc<RefCell<Treatment>>>,
    output_name: Option<String>,

    input_treatment: Option<Rc<RefCell<Treatment>>>,
    input_name: Option<String>,
    
}

impl Connection {
    pub fn new(descriptor: &Rc<ConnectionDescriptor>) -> Self {
        Self {
            descriptor: Rc::clone(descriptor),
            output_treatment: None,
            output_name: None,
            input_treatment: None,
            input_name: None,
        }
    }

    pub fn descriptor(&self) -> &Rc<ConnectionDescriptor> {
        &self.descriptor
    }

    pub fn set_output(&mut self, treatment: &Rc<RefCell<Treatment>>, output: String) {
        self.output_treatment = Some(Rc::clone(treatment));
        self.output_name = Some(output);
    }

    pub fn set_input(&mut self, treatment: &Rc<RefCell<Treatment>>, input: String) {
        self.input_treatment = Some(Rc::clone(treatment));
        self.input_name = Some(input);
    }

    pub fn output_treatment(&self) -> &Option<Rc<RefCell<Treatment>>> {
        &self.output_treatment
    }

    pub fn output_name(&self) -> &Option<String> {
        &self.output_name
    }

    pub fn input_treatment(&self) -> &Option<Rc<RefCell<Treatment>>> {
        &self.input_treatment
    }

    pub fn input_name(&self) -> &Option<String> {
        &self.input_name
    }

    pub fn validate() -> Result<(), LogicError> {
        Ok(())
    }
}
