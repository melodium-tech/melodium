
use std::sync::{Arc, Weak, RwLock};
use super::super::error::LogicError;
use super::super::descriptor::ConnectionDescriptor;
use super::super::descriptor::TreatmentDescriptor;
use super::treatment::Treatment;
use super::sequence::Sequence;

#[derive(Debug)]
pub enum IO {
    Sequence(),
    Treatment(Weak<RwLock<Treatment>>)
}

impl PartialEq for IO {
    
    fn eq(&self, other: &Self) -> bool {
        match self {
            IO::Sequence() => false,
            IO::Treatment(s_t) => {
                match other {
                    IO::Sequence() => false,
                    IO::Treatment(o_t) => {
                        s_t.ptr_eq(o_t)
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct Connection {

    sequence: Weak<RwLock<Sequence>>,

    descriptor: Arc<ConnectionDescriptor>,

    output_treatment: Option<IO>,
    output_name: Option<String>,

    input_treatment: Option<IO>,
    input_name: Option<String>,
    
}

impl Connection {
    pub fn new(sequence: &Arc<RwLock<Sequence>>, descriptor: &Arc<ConnectionDescriptor>) -> Self {
        Self {
            sequence: Arc::downgrade(sequence),
            descriptor: Arc::clone(descriptor),
            output_treatment: None,
            output_name: None,
            input_treatment: None,
            input_name: None,
        }
    }

    pub fn descriptor(&self) -> &Arc<ConnectionDescriptor> {
        &self.descriptor
    }

    /**
     * Assign connection starting point.
     * 
     * Connections works as _treatment output_ -> _treatment input_
     */
    pub fn set_output(&mut self, treatment: &Arc<RwLock<Treatment>>, output: &str) -> Result<(), LogicError> {

        if let Some(output_descriptor) = treatment.read().unwrap().descriptor().outputs().get(output) {

            if output_descriptor.datatype() == self.descriptor.output_type() {

                self.output_treatment = Some(IO::Treatment(Arc::downgrade(treatment)));
                self.output_name = Some(output.to_string());

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

    /**
     * Assign a self input as connection starting point (connection output).
     * 
     * `input_name` is here the name as seen from `Self`, that will be used as output for the connection
     * (connections works as _treatment output_ -> _treatment input_).
     */
    pub fn set_self_output(&mut self, input_name: &str) -> Result<(), LogicError> {
        
        if let Some(input_descriptor) = self.sequence.upgrade().unwrap().read().unwrap()
                                                .descriptor().inputs().get(input_name) {

            if input_descriptor.datatype() == self.descriptor.output_type() {

                self.output_treatment = Some(IO::Sequence());
                self.output_name = Some(input_name.to_string());

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

    /**
     * Assign connection ending point.
     * 
     * Connections works as _treatment output_ -> _treatment input_
     */
    pub fn set_input(&mut self, treatment: &Arc<RwLock<Treatment>>, input: &str) -> Result<(), LogicError> {

        if let Some(input_descriptor) = treatment.read().unwrap().descriptor().inputs().get(input) {

            if input_descriptor.datatype() == self.descriptor.input_type() {

                self.input_treatment = Some(IO::Treatment(Arc::downgrade(treatment)));
                self.input_name = Some(input.to_string());

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

    /**
     * Assign a self ouput as connection ending point (connection input).
     * 
     * `output_name` is here the name as seen from `Self`, that will be used as input for the connection
     * (connections works as _treatment output_ -> _treatment input_).
     */
    pub fn set_self_input(&mut self, output_name: &str) -> Result<(), LogicError> {

        if let Some(output_descriptor) = self.sequence.upgrade().unwrap().read().unwrap()
                                                    .descriptor().outputs().get(output_name) {

            if output_descriptor.datatype() == self.descriptor.input_type() {

                self.input_treatment = Some(IO::Sequence());
                self.input_name = Some(output_name.to_string());

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

        if self.output_name.is_none() {
            return Err(LogicError::connection_output_required())
        }

        if self.input_name.is_none() {
            return Err(LogicError::connection_input_required())
        }

        Ok(())
    }
}
