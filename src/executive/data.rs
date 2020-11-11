
use super::value::Value;

#[derive(Clone, PartialEq)]
pub enum Data {
    Scalar(Value),
    Vector(Vec<Value>),
    Matrix(),
    Collection(),
}
