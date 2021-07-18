
use std::rc::{Rc, Weak};
use std::cell::RefCell;
use std::collections::HashMap;
use std::cmp::Ordering;
use super::super::error::LogicError;
use super::super::descriptor::TreatmentDescriptor;
use super::super::descriptor::model::Model;
use super::sequence::Sequence;
use super::parameter::Parameter;
use super::connection::{IO, Connection};
use super::super::descriptor::ParameterizedDescriptor;
use super::super::descriptor::ParameterDescriptor;
use super::value::Value;
use intertrait::cast::CastRc;

#[derive(Debug)]
pub struct Treatment {

    sequence: Weak<RefCell<Sequence>>,
    descriptor: Rc<dyn TreatmentDescriptor>,
    name: String,
    models: HashMap<String, String>,
    parameters: HashMap<String, Rc<RefCell<Parameter>>>,

    auto_reference: Weak<RefCell<Self>>,
}

impl Treatment {
    pub fn new(sequence: &Rc<RefCell<Sequence>>, descriptor: &Rc<dyn TreatmentDescriptor>, name: &str) -> Rc<RefCell<Self>> {
        let treatment = Rc::<RefCell<Self>>::new(RefCell::new(Self {
            sequence: Rc::downgrade(sequence),
            descriptor: Rc::clone(descriptor),
            name: name.to_string(),
            models: HashMap::with_capacity(descriptor.models().len()),
            parameters: HashMap::with_capacity(descriptor.parameters().len()),
            auto_reference: Weak::new(),
        }));

        treatment.borrow_mut().auto_reference = Rc::downgrade(&treatment);

        treatment
    }

    pub fn descriptor(&self) -> &Rc<dyn TreatmentDescriptor> {
        &self.descriptor
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn add_model(&mut self, parametric_name: &str, local_name: &str) -> Result<(), LogicError> {

        if self.descriptor().models().contains_key(parametric_name) {

            let rc_sequence = self.sequence.upgrade().unwrap();
            let borrowed_sequence = rc_sequence.borrow();

            let mut core_model_descriptor = None;
            if let Some(model_descriptor) = borrowed_sequence.descriptor().models().get(local_name) {
                core_model_descriptor = Some(model_descriptor.core_model());
            }
            else if let Some(model_instanciation) = borrowed_sequence.model_instanciations().get(local_name) {
                core_model_descriptor = Some(model_instanciation.borrow().descriptor().core_model());
            }

            if let Some(model_descriptor) = core_model_descriptor {

                if Rc::ptr_eq(&model_descriptor, self.descriptor().models().get(parametric_name).unwrap()) {
                    self.models.insert(parametric_name.to_string(), local_name.to_string());

                    Ok(())
                }
                else {
                    Err(LogicError::unmatching_model_type())
                }
            }
            else {
                Err(LogicError::unexisting_model())
            }
        }
        else {
            Err(LogicError::unexisting_parametric_model())
        }

    }

    pub fn add_parameter(&mut self, name: &str) -> Result<Rc<RefCell<Parameter>>, LogicError> {
        
        if self.descriptor.parameters().contains_key(name) {
            let parameter = Parameter::new( &(Rc::clone(self.sequence.upgrade().unwrap().borrow().descriptor()) as Rc<dyn ParameterizedDescriptor>), 
                                            &(Rc::clone(&self.descriptor).cast::<dyn ParameterizedDescriptor>().unwrap()),
                                            name
                                        );
            let rc_parameter = Rc::new(RefCell::new(parameter));

            if self.parameters.insert(name.to_string(), Rc::clone(&rc_parameter)).is_none() {
                Ok(rc_parameter)
            }
            else {
                Err(LogicError::multiple_parameter_assignation())
            }
        }
        else {
            Err(LogicError::unexisting_parameter())
        }
    }

    pub fn models(&self) -> &HashMap<String, String> {
        &self.models
    }

    pub fn parameters(&self) -> &HashMap<String, Rc<RefCell<Parameter>>> {
        &self.parameters
    }

    pub fn validate(&self) -> Result<(), LogicError> {

        for (_, param) in &self.parameters {
            param.borrow().validate()?;
        }
        
        // Check if all parameters are filled.
        let unset_params: Vec<&ParameterDescriptor> = self.descriptor.parameters().iter().filter_map(
            |(core_param_name, core_param)|
            if self.parameters.contains_key(core_param_name) {
                None
            }
            else if core_param.default().is_some() {
                None
            }
            else {
                Some(core_param)
            }
        ).collect();

        if !unset_params.is_empty() {
            return Err(LogicError::unset_parameter());
        }

        // Check if all models are filled
        let unset_models: Vec<&String> = self.descriptor.models().iter().filter_map(
            |(model_name, _)|
            if self.models.contains_key(model_name) {
                None
            }
            else {
                Some(model_name)
            }
        ).collect();

        if !unset_models.is_empty() {
            return Err(LogicError::unset_model());
        }

        // Check if context values refers to available context.
        let rc_sequence = self.sequence.upgrade().unwrap();
        let borrowed_sequence = rc_sequence.borrow();
        if let Some(_unavailable_context) = self.parameters.iter().find(
            |(_param_name, param)|
            match param.borrow().value().as_ref().unwrap() {
                Value::Context((name, _var)) => 
                    !borrowed_sequence.descriptor().requirements().contains_key(name),
                _ => false
            }
        ) {
            return Err(LogicError::unavailable_context())
        }

        Ok(())
    }

    pub fn level(&self) -> usize {

        let rc_sequence = self.sequence.upgrade().unwrap();
        let borrowed_sequence = rc_sequence.borrow();
        let all_connections = borrowed_sequence.connections();

        // We initialize the considered connection by taking only the ones were the current treatment
        // is set as input (end point of the connection).
        let mut considered_connections: Vec<Rc<RefCell<Connection>>> = all_connections.iter().filter_map(
            |raw_conn|
            if let Some(treatment) = raw_conn.borrow().input_treatment() {
                match treatment {
                    IO::Sequence() => None,
                    IO::Treatment(t) => {
                        // We want the input (end point) to be the current treatment, and the output (start point) to not be 'Self'-sequence.
                        if self.auto_reference.ptr_eq(t) && raw_conn.borrow().output_treatment() != &Some(IO::Sequence()) {
                            Some(Rc::clone(&raw_conn))
                        }
                        else {
                            None
                        }
                    }
                }
            }
            else {
                None
            }
        ).collect();

        let mut level = 0;

        while considered_connections.len() > 0 {

            level += 1;

            // We retain only connections that have as input (end point) a treatment which is an ancestor
            // of the current treatment (output, start point).
            let next_considered_connections = all_connections.iter().filter_map(
                |raw_conn|

                if considered_connections.iter().any(
                    |conn|
                    conn.borrow().output_treatment() == raw_conn.borrow().input_treatment()
                ) {
                    Some(Rc::clone(&raw_conn))
                }
                else {
                    None
                }
            ).collect();

            considered_connections = next_considered_connections;
        }

        level
    }
}

impl PartialOrd for Treatment {
    
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {

        if self.sequence.ptr_eq(&other.sequence) {

            let self_level = self.level();
            let other_level = other.level();

            self_level.partial_cmp(&other_level)
        }
        else {
            None
        }
    }
}

impl PartialEq for Treatment {
    
    fn eq(&self, other: &Self) -> bool {
        if self.sequence.ptr_eq(&other.sequence) {
            self.level() == other.level()
        }
        else {
            false
        }
    }
}
