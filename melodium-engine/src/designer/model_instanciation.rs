use super::{Parameter, Scope, Treatment, Value};
use crate::error::LogicError;
use core::fmt::Debug;
use melodium_common::descriptor::{
    Model as ModelDescriptor, Parameter as ParameterDescriptor, Variability,
};
use std::collections::HashMap;
use std::sync::{Arc, RwLock, Weak};

#[derive(Debug)]
pub struct ModelInstanciation {
    host_treatment: Weak<RwLock<Treatment>>,
    descriptor: Weak<dyn ModelDescriptor>,
    name: String,
    parameters: HashMap<String, Arc<RwLock<Parameter>>>,
}

impl ModelInstanciation {
    pub fn new(
        host_treatment: &Arc<RwLock<Treatment>>,
        descriptor: &Arc<dyn ModelDescriptor>,
        name: &str,
    ) -> Self {
        Self {
            host_treatment: Arc::downgrade(host_treatment),
            descriptor: Arc::downgrade(descriptor),
            name: name.to_string(),
            parameters: HashMap::with_capacity(descriptor.parameters().len()),
        }
    }

    pub fn descriptor(&self) -> Arc<dyn ModelDescriptor> {
        self.descriptor.upgrade().unwrap()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn add_parameter(&mut self, name: &str) -> Result<Arc<RwLock<Parameter>>, LogicError> {
        if self.descriptor().parameters().contains_key(name) {
            let parameter = Parameter::new(
                &(self.host_treatment.upgrade().unwrap() as Arc<RwLock<dyn Scope>>),
                &self.descriptor().as_parameterized(),
                name,
            );
            let rc_parameter = Arc::new(RwLock::new(parameter));

            if self
                .parameters
                .insert(name.to_string(), Arc::clone(&rc_parameter))
                .is_none()
            {
                Ok(rc_parameter)
            } else {
                Err(LogicError::multiple_parameter_assignation())
            }
        } else {
            Err(LogicError::unexisting_parameter())
        }
    }

    pub fn remove_parameter(&mut self, name: &str) -> Result<bool, LogicError> {
        if let Some(_) = self.parameters.remove(name) {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn parameters(&self) -> &HashMap<String, Arc<RwLock<Parameter>>> {
        &self.parameters
    }

    pub fn validate(&self) -> Result<(), LogicError> {
        for (_, param) in &self.parameters {
            param.read().unwrap().validate()?;
        }

        let descriptor = self.descriptor();

        // Check if all model parameters are filled.
        let unset_params: Vec<&ParameterDescriptor> = descriptor
            .parameters()
            .iter()
            .filter_map(|(core_param_name, core_param)| {
                if self.parameters.contains_key(core_param_name) {
                    None
                } else if core_param.default().is_some() {
                    None
                } else {
                    Some(core_param)
                }
            })
            .collect();

        if !unset_params.is_empty() {
            return Err(LogicError::unset_parameter());
        }

        // Check all parameters does not refers to a context.
        if let Some(_forbidden_context) = self.parameters.iter().find(|&(_param_name, param)| {
            matches!(param.read().unwrap().value(), Some(Value::Context { .. }))
        }) {
            return Err(LogicError::no_context());
        }

        // Check all parameters are const.
        if let Some(_forbidden_var) = self.parameters.iter().find(|&(param_name, param)| {
            *param
                .read()
                .unwrap()
                .parent_descriptor()
                .upgrade()
                .unwrap()
                .parameters()
                .get(param_name)
                .unwrap()
                .variability()
                != Variability::Const
        }) {
            return Err(LogicError::model_instanciation_const_only());
        }

        Ok(())
    }
}
