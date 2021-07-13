
use std::rc::{Rc, Weak};
use std::cell::RefCell;
use std::collections::HashMap;
use super::super::error::LogicError;
use super::super::descriptor::TreatmentDescriptor;
use super::super::descriptor::model::Model;
use super::sequence::Sequence;
use super::parameter::Parameter;
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
}

impl Treatment {
    pub fn new(sequence: &Rc<RefCell<Sequence>>, descriptor: &Rc<dyn TreatmentDescriptor>, name: &str) -> Self {
        Self {
            sequence: Rc::downgrade(sequence),
            descriptor: Rc::clone(descriptor),
            name: name.to_string(),
            models: HashMap::with_capacity(descriptor.models().len()),
            parameters: HashMap::with_capacity(descriptor.parameters().len()),
        }
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

        // TODO check if all models are filled

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
}
