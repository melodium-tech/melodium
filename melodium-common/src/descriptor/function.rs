use super::{DataType, DescribedType, Documented, Identified, OrderedParameterized};
use crate::executive::Value;
use core::fmt::{Debug, Display};
use downcast_rs::{impl_downcast, DowncastSync};
use std::sync::Arc;

pub trait Function:
    Identified + Documented + OrderedParameterized + DowncastSync + Display + Debug + Send + Sync
{
    fn return_type(&self) -> &DescribedType;
    fn function(&self) -> fn(Vec<Value>) -> Value;
    fn as_identified(&self) -> Arc<dyn Identified>;
    fn as_ordered_parameterized(&self) -> Arc<dyn OrderedParameterized>;
}
impl_downcast!(sync Function);
