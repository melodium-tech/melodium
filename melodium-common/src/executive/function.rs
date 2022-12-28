use crate::descriptor::Function as FunctionDescriptor;
use crate::executive::Value;
use std::sync::Arc;

pub trait Function {
    fn descriptor(&self) -> Arc<dyn FunctionDescriptor>;

    fn execute(&self, parameters: Vec<Value>) -> Value;
}
