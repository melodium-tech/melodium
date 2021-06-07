
use super::value::Value;

#[derive(Clone, PartialEq, Debug)]
pub enum Data {
    Block(Value),
    Stream(Vec<Value>),
}
