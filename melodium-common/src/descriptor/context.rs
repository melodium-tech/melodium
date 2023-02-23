use super::{DataType, Documented, Identified};
use core::fmt::{Debug, Display};
use std::collections::HashMap;

pub trait Context: Display + Identified + Documented + Debug + Send + Sync {
    fn name(&self) -> &str;
    fn values(&self) -> &HashMap<String, DataType>;
}
