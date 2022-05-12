
use std::sync::Arc;
use super::value::Value;
use crate::logic::descriptor::CoreFunctionDescriptor;

pub trait Function {

    fn descriptor(&self) -> Arc<CoreFunctionDescriptor>;

    fn execute(&self, parameters: Vec<Value>) -> Value;
}

