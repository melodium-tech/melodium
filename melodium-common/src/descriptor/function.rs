
use core::fmt::{Debug, Display};
use std::sync::Arc;
use downcast_rs::{DowncastSync, impl_downcast};
use crate::executive::Value;
use super::{Identified, OrderedParameterized, Documented, DataType};

pub trait Function: Identified + Documented + OrderedParameterized + DowncastSync + Display + Debug + Send + Sync {
    fn return_type(&self) -> &DataType;
    fn function(&self) -> fn(Vec<Value>) -> Value;
    fn as_identified(&self) -> Arc<dyn Identified>;
    fn as_ordered_parameterized(&self) -> Arc<dyn OrderedParameterized>;
}
impl_downcast!(sync Function);
