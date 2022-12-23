
use std::sync::Arc;
use crate::descriptor::Function as FunctionDescriptor;
use crate::executive::Value;

pub trait Function {

    fn descriptor(&self) -> Arc<dyn FunctionDescriptor>;

    fn execute(&self, parameters: Vec<Value>) -> Value;
}
