
use super::value::Value;

#[derive(Clone, PartialEq, Debug)]
pub enum Data {
    Scalar(Value),
    Vector(Vec<Value>),
    Matrix(),
    Collection(),
}
