
use std::fmt::Debug;
use downcast_rs::{DowncastSync, impl_downcast};
use super::identified::Identified;
use super::ordered_parameterized::OrderedParameterized;
use super::datatype::DataType;

pub trait Function: Identified + OrderedParameterized + DowncastSync + Debug + Send + Sync {
    fn return_type(&self) -> &DataType;
}
impl_downcast!(sync Function);
