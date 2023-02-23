use melodium_common::descriptor::{Context, Function};
use melodium_common::executive::Value as ExecutiveValue;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub enum Value {
    Raw(ExecutiveValue),
    Variable(String),
    Context(Arc<dyn Context>, String),
    Function(Arc<dyn Function>, Vec<Value>),
}
