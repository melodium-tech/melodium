
use std::fmt::*;
use std::sync::{Arc, Weak};
use super::identified::Identified;
use super::identifier::Identifier;
use super::function::Function;
use super::ordered_parameterized::OrderedParameterized;
use super::parameter::Parameter;
use super::datatype::DataType;
use crate::executive::value::Value;

#[derive(Debug)]
pub struct CoreFunction {
    identifier: Identifier,
    parameters: Vec<Parameter>,
    return_type: DataType,
    function: fn(Vec<Value>) -> Value,
    auto_reference: Weak<Self>,
}

impl CoreFunction {
    pub fn new(identifier: Identifier, parameters: Vec<Parameter>, return_type: DataType, function: fn(Vec<Value>) -> Value) -> Arc<Self> {
        Arc::new_cyclic(|me| Self {
            identifier,
            parameters,
            return_type,
            function,
            auto_reference: me.clone(),
        })
    }
}

impl Function for CoreFunction {

    fn return_type(&self) -> &DataType {
        &self.return_type
    }

    fn function(&self) -> fn(Vec<Value>) -> Value {
        self.function
    }
}

impl Identified for CoreFunction {
    fn identifier(&self) -> &Identifier {
        &self.identifier
    }
}

impl OrderedParameterized for CoreFunction {

    fn parameters(&self) -> &Vec<Parameter> {
        &self.parameters
    }

    fn as_ordered_parameterized(&self) -> Arc<dyn OrderedParameterized> {
        self.auto_reference.upgrade().unwrap()
    }
}

