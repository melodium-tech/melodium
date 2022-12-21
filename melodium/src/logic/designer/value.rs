
use std::sync::Arc;
use crate::executive::value::Value as ExecutiveValue;
use super::super::descriptor::FunctionDescriptor;

#[derive(Clone, Debug)]
pub enum Value {
    Raw(ExecutiveValue),
    Variable(String),
    Context((String, String)),
    Function(Arc<dyn FunctionDescriptor>, Vec<Value>),
}


