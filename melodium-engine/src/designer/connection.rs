
use std::sync::{Arc, Weak, RwLock};
use crate::error::LogicError;
use melodium_common::descriptor::{Input, Output};
use super::TreatmentInstanciation;

#[derive(Debug)]
pub enum IO {
    Sequence(),
    Treatment(Weak<RwLock<TreatmentInstanciation>>)
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

    output_treatment: Option<IO>,
    output_name: Option<String>,

    input_treatment: Option<IO>,
    input_name: Option<String>,
    
}

impl Connection {
    pub fn new() -> Self {
        Self {
            output_treatment: None,
            output_name: None,
            input_treatment: None,
            input_name: None,
        }
    }

    /**
     * Assign connection starting point.
     * 
     * Connections works as _treatment output_ -> _treatment input_
     */
    pub fn set_output(&mut self, treatment: &Arc<RwLock<TreatmentInstanciation>>, output: &Output) -> Result<(), LogicError> {

        if output.datatype() == self.descriptor.output_type() {

            self.output_treatment = Some(IO::Treatment(Arc::downgrade(treatment)));
            self.output_name = Some(output.name().to_string());

            Ok(())
        }
        else {
            Err(LogicError::connection_output_unmatching_datatype())
        }
    }

    /**
     * Assign a self input as connection starting point (connection output).
     * 
     * `input` is seen from `Self`, that will be used as output for the connection
     * (connections works as _treatment output_ -> _treatment input_).
     */
    pub fn set_self_output(&mut self, input: &Input) -> Result<(), LogicError> {
        
        if input.datatype() == self.descriptor.output_type() {

            self.output_treatment = Some(IO::Sequence());
            self.output_name = Some(input.name().to_string());

            Ok(())
        }
        else {
            Err(LogicError::connection_output_unmatching_datatype())
        }
    }

    /**
     * Assign connection ending point.
     * 
     * Connections works as _treatment output_ -> _treatment input_
     */
    pub fn set_input(&mut self, treatment: &Arc<RwLock<TreatmentInstanciation>>, input: &Input) -> Result<(), LogicError> {

        if input.datatype() == self.descriptor.input_type() {

            self.input_treatment = Some(IO::Treatment(Arc::downgrade(treatment)));
            self.input_name = Some(input.name().to_string());

            Ok(())
        }
        else {
            Err(LogicError::connection_input_unmatching_datatype())
        }
    }

    /**
     * Assign a self ouput as connection ending point (connection input).
     * 
     * `output` is seen from `Self`, that will be used as input for the connection
     * (connections works as _treatment output_ -> _treatment input_).
     */
    pub fn set_self_input(&mut self, output: &Output) -> Result<(), LogicError> {

        if output.datatype() == self.descriptor.input_type() {

            self.input_treatment = Some(IO::Sequence());
            self.input_name = Some(output.name().to_string());

            Ok(())
        }
        else {
            Err(LogicError::connection_input_unmatching_datatype())
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
