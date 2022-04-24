
use std::sync::{Arc, Weak};
use super::super::error::LogicError;
use super::super::descriptor::{ParameterizedDescriptor, VariabilityDescriptor};
use super::value::Value;
use super::super::contexts::Contexts;

#[derive(Debug)]
pub struct Parameter {

    scope: Weak<dyn ParameterizedDescriptor>,
    parent_descriptor: Weak<dyn ParameterizedDescriptor>,
    name: String,
    value: Option<Value>,
}

impl Parameter {
    pub fn new(scope: &Arc<dyn ParameterizedDescriptor>, parent_descriptor: &Arc<dyn ParameterizedDescriptor>, name: &str) -> Self {
        Self {
            scope: Arc::downgrade(scope),
            parent_descriptor: Arc::downgrade(parent_descriptor),
            name: name.to_string(),
            value: None,
        }
    }

    pub fn scope(&self) -> &Weak<dyn ParameterizedDescriptor> {
        &self.scope
    }

    pub fn parent_descriptor(&self) -> &Weak<dyn ParameterizedDescriptor> {
        &self.parent_descriptor
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_value(&mut self, value: Value) -> Result<(), LogicError> {
        
        match &value {
            Value::Raw(data) => {
                if !self.parent_descriptor.upgrade().unwrap().parameters().get(&self.name).unwrap().datatype().is_compatible(data) {
                    return Err(LogicError::unmatching_datatype())
                }
            },
            Value::Variable(name) => {

                if let Some(scope_variable) = self.scope.upgrade().unwrap().parameters().get(name) {

                    if *self.parent_descriptor.upgrade().unwrap().parameters().get(&self.name).unwrap().variability() == VariabilityDescriptor::Const
                    && *scope_variable.variability() != VariabilityDescriptor::Const {
                        // return truc const non respectée
                    }

                    if scope_variable.datatype() != self.parent_descriptor.upgrade().unwrap().parameters().get(&self.name).unwrap().datatype() {
                        return Err(LogicError::unmatching_datatype())
                    }
                }
                else {
                    return Err(LogicError::unexisting_variable())
                }
            },
            Value::Context((context, name)) => {

                if *self.parent_descriptor.upgrade().unwrap().parameters().get(&self.name).unwrap().variability() == VariabilityDescriptor::Const {
                    // return truc const non respectée
                }

                if let Some(context_descriptor) = Contexts::get(context) {

                    if let Some(context_variable_datatype) = context_descriptor.values().get(name) {
                        
                        if context_variable_datatype != self.parent_descriptor.upgrade().unwrap().parameters().get(&self.name).unwrap().datatype() {
                            return Err(LogicError::unmatching_datatype())
                        }
                    }
                    else {
                        return Err(LogicError::unexisting_context_variable())
                    }
                }
                else {
                    return Err(LogicError::unexisting_context())
                }
            }
        }

        self.value = Some(value);

        Ok(())
    }

    pub fn value(&self) -> &Option<Value> {
        &self.value
    }

    pub fn validate(&self) -> Result<(), LogicError> {
        
        if let Some(_v) = &self.value {
            Ok(())
        }
        else {
            Err(LogicError::no_value())
        }
    }
}
