
use std::fmt::Debug;
use downcast_rs::{DowncastSync, impl_downcast};
use super::identified::Identified;
use super::documented::Documented;
use super::ordered_parameterized::OrderedParameterized;
use super::datatype::DataType;
use crate::executive::value::Value;

pub trait Function: Identified + Documented + OrderedParameterized + DowncastSync + Debug + Send + Sync {
    fn return_type(&self) -> &DataType;
    fn function(&self) -> fn(Vec<Value>) -> Value;
}
impl_downcast!(sync Function);
