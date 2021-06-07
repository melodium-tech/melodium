
use crate::executive::value::Value as ExecutiveValue;

#[derive(Clone, Debug)]
pub enum Value {
    Raw(ExecutiveValue),
    Variable(String),
    Context((String, String)),
}
