
use std::sync::{Arc, Weak, RwLock};
use super::super::error::LogicError;
use super::super::descriptor::{ParameterizedDescriptor, VariabilityDescriptor};
use super::super::contexts::Contexts;
use super::super::descriptor::FunctionDescriptor;
use super::scope::Scope;
use super::value::Value;

#[derive(Debug)]
pub struct Parameter {

    scope: Weak<RwLock<dyn Scope>>,
    parent_descriptor: Weak<dyn ParameterizedDescriptor>,
    name: String,
    value: Option<Value>,
}

impl Parameter {
    pub fn new(scope: &Arc<RwLock<dyn Scope>>, parent_descriptor: &Arc<dyn ParameterizedDescriptor>, name: &str) -> Self {
        Self {
            scope: Arc::downgrade(scope),
            parent_descriptor: Arc::downgrade(parent_descriptor),
            name: name.to_string(),
            value: None,
        }
    }

    pub fn scope(&self) -> &Weak<RwLock<dyn Scope>> {
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

                if let Some(scope_variable) = self.scope.upgrade().unwrap().read().unwrap().descriptor().parameters().get(name) {

                    if *self.parent_descriptor.upgrade().unwrap().parameters().get(&self.name).unwrap().variability() == VariabilityDescriptor::Const
                    && *scope_variable.variability() != VariabilityDescriptor::Const {
                        return Err(LogicError::const_required_var_provided())
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
                    return Err(LogicError::const_required_context_provided())
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
            },
            Value::Function(descriptor, parameters) => {

                let variability = self.check_function_return(descriptor, parameters)?;

                if *self.parent_descriptor.upgrade().unwrap().parameters().get(&self.name).unwrap().variability() == VariabilityDescriptor::Const
                    && variability != VariabilityDescriptor::Const {
                    return Err(LogicError::const_required_function_returns_var())
                }
            }
        }

        self.value = Some(value);

        Ok(())
    }

    fn check_function_return(&self, descriptor: &Arc<dyn FunctionDescriptor>, parameters: &Vec<Value>) -> Result<VariabilityDescriptor, LogicError> {

        let mut variability = VariabilityDescriptor::Const;

        if descriptor.parameters().len() != parameters.len() {
            return Err(LogicError::unmatching_number_of_parameters())
        }

        for i in 0..descriptor.parameters().len() {

            let descriptor = &descriptor.parameters()[i];
            match &parameters[i] {
                Value::Raw(data) => {
                    if !descriptor.datatype().is_compatible(&data) {
                        return Err(LogicError::unmatching_datatype())
                    }
                },
                Value::Variable(name) => {
                    if let Some(scope_variable) = self.scope.upgrade().unwrap().read().unwrap().descriptor().parameters().get(name) {
    
                        if *scope_variable.variability() != VariabilityDescriptor::Const {
                            variability = VariabilityDescriptor::Var;
                        }
    
                        if scope_variable.datatype() != descriptor.datatype() {
                            return Err(LogicError::unmatching_datatype())
                        }
                    }
                    else {
                        return Err(LogicError::unexisting_variable())
                    }
                },
                Value::Context((context, name)) => {

                    variability = VariabilityDescriptor::Var;
    
                    if let Some(context_descriptor) = Contexts::get(context) {
    
                        if let Some(context_variable_datatype) = context_descriptor.values().get(name) {
                            
                            if context_variable_datatype != descriptor.datatype() {
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
                },
                Value::Function(descriptor, parameters) => {
                    if self.check_function_return(descriptor, parameters)? != VariabilityDescriptor::Const {
                        variability = VariabilityDescriptor::Var
                    }
                },
            }
        }

        Ok(variability)
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
