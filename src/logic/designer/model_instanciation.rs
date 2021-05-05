
use std::rc::{Rc, Weak};
use std::cell::RefCell;
use std::collections::HashMap;
use super::super::error::LogicError;
use super::super::descriptor::ModelDescriptor;
use super::sequence::Sequence;
use super::parameter::Parameter;
use super::super::descriptor::ParameterizedDescriptor;
use super::super::descriptor::ParameterDescriptor;
use super::value::Value;
use intertrait::cast::CastRc;

#[derive(Debug)]
pub struct ModelInstanciation {

    sequence: Weak<RefCell<Sequence>>,
    descriptor: Rc<dyn ModelDescriptor>,
    name: String,
    parameters: HashMap<String, Rc<RefCell<Parameter>>>,
}

impl ModelInstanciation {
    pub fn new(sequence: &Rc<RefCell<Sequence>>, descriptor: &Rc<dyn ModelDescriptor>, name: &str) -> Self {
        Self {
            sequence: Rc::downgrade(sequence),
            descriptor: Rc::clone(descriptor),
            name: name.to_string(),
            parameters: HashMap::with_capacity(descriptor.parameters().len()),
        }
    }

    pub fn descriptor(&self) -> &Rc<dyn ModelDescriptor> {
        &self.descriptor
    }

    pub fn name(&self) -> &str {
        &self.name
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

    pub fn parameters(&self) -> &HashMap<String, Rc<RefCell<Parameter>>> {
        &self.parameters
    }

    pub fn validate(&self) -> Result<(), LogicError> {
        
        for (_, param) in &self.parameters {
            param.borrow().validate()?;
        }

        // Check if all model parameters are filled.
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

        // Check all parameters does not refers to a context.
        if let Some(_forbidden_context) = self.parameters.iter().find(|&(_param_name, param)| !matches!(param.borrow().value(), Some(Value::Context{..}))) {
            return Err(LogicError::no_context())
        }

        Ok(())
    }

}