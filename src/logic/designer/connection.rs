
use std::rc::{Rc, Weak};
use std::cell::RefCell;
use super::super::error::LogicError;
use super::super::ConnectionDescriptor;
use super::treatment::Treatment;
use super::sequence::Sequence;

pub struct Connection {

    sequence: Weak<RefCell<Sequence>>,

    descriptor: Rc<ConnectionDescriptor>,

    output_treatment: Weak<RefCell<Treatment>>,
    output_name: Option<String>,

    input_treatment: Weak<RefCell<Treatment>>,
    input_name: Option<String>,
    
}

impl Connection {
    pub fn new(sequence: &Rc<RefCell<Sequence>>, descriptor: &Rc<ConnectionDescriptor>) -> Self {
        Self {
            sequence: Rc::downgrade(sequence),
            descriptor: Rc::clone(descriptor),
            output_treatment: Weak::new(),
            output_name: None,
            input_treatment: Weak::new(),
            input_name: None,
        }
    }

    pub fn descriptor(&self) -> &Rc<ConnectionDescriptor> {
        &self.descriptor
    }

    pub fn set_output(&mut self, treatment: &Rc<RefCell<Treatment>>, output: String) {
        self.output_treatment = Rc::downgrade(treatment);
        self.output_name = Some(output);
    }

    pub fn set_input(&mut self, treatment: &Rc<RefCell<Treatment>>, input: String) {
        self.input_treatment = Rc::downgrade(treatment);
        self.input_name = Some(input);
    }

    pub fn output_treatment(&self) -> Option<Rc<RefCell<Treatment>>> {
        self.output_treatment.upgrade()
    }

    pub fn output_name(&self) -> &Option<String> {
        &self.output_name
    }

    pub fn input_treatment(&self) -> Option<Rc<RefCell<Treatment>>> {
        self.input_treatment.upgrade()
    }

    pub fn input_name(&self) -> &Option<String> {
        &self.input_name
    }

    pub fn validate() -> Result<(), LogicError> {
        Ok(())
    }
}
