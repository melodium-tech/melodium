
use std::sync::{Arc, Weak, RwLock};
use crate::error::LogicError;
use super::{Scope, Value};
use melodium_common::descriptor::{Parameterized, Variability, Function};

#[derive(Debug)]
pub struct Parameter {

    scope: Weak<RwLock<dyn Scope>>,
    parent_descriptor: Weak<dyn Parameterized>,
    name: String,
    value: Option<Value>,
}

impl Parameter {
    pub fn new(scope: &Arc<RwLock<dyn Scope>>, parent_descriptor: &Arc<dyn Parameterized>, name: &str) -> Self {
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

    pub fn parent_descriptor(&self) -> &Weak<dyn Parameterized> {
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

                    if *self.parent_descriptor.upgrade().unwrap().parameters().get(&self.name).unwrap().variability() == Variability::Const
                    && *scope_variable.variability() != Variability::Const {
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
            Value::Context(context, name) => {

                if *self.parent_descriptor.upgrade().unwrap().parameters().get(&self.name).unwrap().variability() == Variability::Const {
                    return Err(LogicError::const_required_context_provided())
                }

                if let Some(context_variable_datatype) = context.values().get(name) {
                        
                    if context_variable_datatype != self.parent_descriptor.upgrade().unwrap().parameters().get(&self.name).unwrap().datatype() {
                        return Err(LogicError::unmatching_datatype())
                    }
                }
                else {
                    return Err(LogicError::unexisting_context_variable())
                }
            },
            Value::Function(descriptor, parameters) => {

                let variability = self.check_function_return(descriptor, parameters)?;

                if *self.parent_descriptor.upgrade().unwrap().parameters().get(&self.name).unwrap().variability() == Variability::Const
                    && variability != Variability::Const {
                    return Err(LogicError::const_required_function_returns_var())
                }
            }
        }

        self.value = Some(value);

        Ok(())
    }

    fn check_function_return(&self, descriptor: &Arc<dyn Function>, parameters: &Vec<Value>) -> Result<Variability, LogicError> {

        let mut variability = Variability::Const;

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
    
                        if *scope_variable.variability() != Variability::Const {
                            variability = Variability::Var;
                        }
    
                        if scope_variable.datatype() != descriptor.datatype() {
                            return Err(LogicError::unmatching_datatype())
                        }
                    }
                    else {
                        return Err(LogicError::unexisting_variable())
                    }
                },
                Value::Context(context, name) => {

                    variability = Variability::Var;
    
                    if let Some(context_variable_datatype) = context.values().get(name) {
                            
                        if context_variable_datatype != descriptor.datatype() {
                            return Err(LogicError::unmatching_datatype())
                        }
                    }
                    else {
                        return Err(LogicError::unexisting_context_variable())
                    }
                },
                Value::Function(descriptor, parameters) => {
                    if self.check_function_return(descriptor, parameters)? != Variability::Const {
                        variability = Variability::Var
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
