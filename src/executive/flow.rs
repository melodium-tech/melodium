
use super::value::Value;

#[derive(Clone, PartialEq, Debug)]
pub enum Flow {
    Block(Value),
    Stream(Vec<Value>),
}
