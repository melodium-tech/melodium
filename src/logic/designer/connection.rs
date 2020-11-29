
use std::rc::{Rc, Weak};
use std::cell::RefCell;
use super::super::error::LogicError;
use super::super::ConnectionDescriptor;
use super::super::TreatmentDescriptor;
use super::treatment::Treatment;
use super::sequence::Sequence;

pub enum IO {
    Sequence(),
    Treatement(Weak<RefCell<Treatment>>)
}

pub struct Connection {

    sequence: Weak<RefCell<Sequence>>,

    descriptor: Rc<ConnectionDescriptor>,

    output_treatment: Option<IO>,
    output_name: Option<String>,

    input_treatment: Option<IO>,
    input_name: Option<String>,
    
}

impl Connection {
    pub fn new(sequence: &Rc<RefCell<Sequence>>, descriptor: &Rc<ConnectionDescriptor>) -> Self {
        Self {
            sequence: Rc::downgrade(sequence),
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

    pub fn set_output(&mut self, treatment: &Rc<RefCell<Treatment>>, output: Option<&str>) -> Result<(), LogicError> {

        if output.is_none() {
            if self.descriptor.output_type().is_none() {
                self.output_treatment = Some(IO::Treatement(Rc::downgrade(treatment)));
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

                self.output_treatment = Some(IO::Treatement(Rc::downgrade(treatment)));
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
                self.output_treatment = Some(IO::Sequence());
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

                self.output_treatment = Some(IO::Sequence());
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
                self.input_treatment = Some(IO::Treatement(Rc::downgrade(treatment)));
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

                self.input_treatment = Some(IO::Treatement(Rc::downgrade(treatment)));
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
                self.input_treatment = Some(IO::Sequence());
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

                self.input_treatment = Some(IO::Sequence());
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

    pub fn output_treatment(&self) -> &Option<IO> {
        &self.output_treatment
    }

    pub fn output_name(&self) -> &Option<String> {
        &self.output_name
    }

    pub fn input_treatment(&self) -> &Option<IO> {
        &self.input_treatment
    }

    pub fn input_name(&self) -> &Option<String> {
        &self.input_name
    }

    pub fn validate(&self) -> Result<(), LogicError> {

        if self.output_treatment.is_none() {
            return Err(LogicError::connection_output_not_set())
        }

        if self.input_treatment.is_none() {
            return Err(LogicError::connection_input_not_set())
        }

        // Check if descriptor require an output or not, then if one is assigned.
        if let Some(_output) = self.descriptor.output_type() {
            if self.output_name.is_none() {
                return Err(LogicError::connection_output_required())
            }
        }
        else {
            if self.output_name.is_some() {
                return Err(LogicError::connection_output_forbidden())
            }
        }

        // Check if descriptor require an input or not, then if one is assigned.
        if let Some(_input) = self.descriptor.input_type() {
            if self.input_name.is_none() {
                return Err(LogicError::connection_input_required())
            }
        }
        else {
            if self.input_name.is_some() {
                return Err(LogicError::connection_input_forbidden())
            }
        }

        Ok(())
    }
}
