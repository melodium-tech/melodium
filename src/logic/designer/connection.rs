
use std::rc::{Rc, Weak};
use std::cell::RefCell;
use super::super::error::LogicError;
use super::super::ConnectionDescriptor;
use super::super::TreatmentDescriptor;
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

    pub fn set_output(&mut self, treatment: &Rc<RefCell<Treatment>>, output: Option<&str>) -> Result<(), LogicError> {

        if output.is_none() {
            if self.descriptor.output_type().is_none() {
                self.output_treatment = Rc::downgrade(treatment);
                self.output_name = None;

                Ok(())
            }
            else {
                Err(LogicError::connection_output_required())
            }
        }
        else if let Some(output_descriptor) = treatment.borrow().descriptor().outputs().get(output.unwrap()) {

            if self.descriptor.output_type().is_none() {
                Err(LogicError::connection_output_forbidden())
            }
            else if output_descriptor.datatype() == self.descriptor.output_type().as_ref().unwrap() {

                self.output_treatment = Rc::downgrade(treatment);
                self.output_name = output.map(String::from);

                Ok(())
            }
            else {
                Err(LogicError::connection_output_unmatching_datatype())
            }
        }
        else {
            Err(LogicError::connection_output_not_found())
        }
    }

    pub fn set_self_output(&mut self, input_name: Option<&str>) -> Result<(), LogicError> {
        
        if input_name.is_none() {
            if self.descriptor.output_type().is_none() {
                self.output_treatment = Weak::default();
                self.output_name = None;

                Ok(())
            }
            else {
                Err(LogicError::connection_output_required())
            }
        }
        else if let Some(input_descriptor) = self.sequence.upgrade().unwrap().borrow()
                                                .descriptor().inputs().get(input_name.unwrap()) {

            if self.descriptor.output_type().is_none() {
                Err(LogicError::connection_output_forbidden())
            }
            else if input_descriptor.datatype() == self.descriptor.output_type().as_ref().unwrap() {

                self.output_treatment = Weak::default();
                self.output_name = input_name.map(String::from);

                Ok(())
            }
            else {
                Err(LogicError::connection_output_unmatching_datatype())
            }
        }
        else {
            Err(LogicError::connection_output_not_found())
        }
    }

    pub fn set_input(&mut self, treatment: &Rc<RefCell<Treatment>>, input: Option<&str>) -> Result<(), LogicError> {

        if input.is_none() {
            if self.descriptor.input_type().is_none() {
                self.input_treatment = Rc::downgrade(treatment);
                self.input_name = None;

                Ok(())
            }
            else {
                Err(LogicError::connection_input_required())
            }
        }
        else if let Some(input_descriptor) = treatment.borrow().descriptor().inputs().get(input.unwrap()) {

            if self.descriptor.input_type().is_none() {
                Err(LogicError::connection_input_forbidden())
            }
            else if input_descriptor.datatype() == self.descriptor.input_type().as_ref().unwrap() {

                self.input_treatment = Rc::downgrade(treatment);
                self.input_name = input.map(String::from);

                Ok(())
            }
            else {
                Err(LogicError::connection_input_unmatching_datatype())
            }
        }
        else {
            Err(LogicError::connection_input_not_found())
        }
    }

    pub fn set_self_input(&mut self, output_name: Option<&str>) -> Result<(), LogicError> {

        if output_name.is_none() {
            if self.descriptor.input_type().is_none() {
                self.input_treatment = Weak::default();
                self.input_name = None;

                Ok(())
            }
            else {
                Err(LogicError::connection_input_required())
            }
        }
        else if let Some(output_descriptor) = self.sequence.upgrade().unwrap().borrow()
                                                    .descriptor().outputs().get(output_name.unwrap()) {

            if self.descriptor.input_type().is_none() {
                Err(LogicError::connection_input_forbidden())
            }
            else if output_descriptor.datatype() == self.descriptor.input_type().as_ref().unwrap() {

                self.input_treatment = Weak::default();
                self.input_name = output_name.map(String::from);

                Ok(())
            }
            else {
                Err(LogicError::connection_input_unmatching_datatype())
            }
        }
        else {
            Err(LogicError::connection_input_not_found())
        }
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
