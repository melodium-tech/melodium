
use std::sync::Arc;
use melodium_common::descriptor::Function;
use melodium_common::executive::Value as ExecutiveValue;

#[derive(Clone, Debug)]
pub enum Value {
    Raw(ExecutiveValue),
    Variable(String),
    Context((String, String)),
    Function(Arc<dyn Function>, Vec<Value>),
}


