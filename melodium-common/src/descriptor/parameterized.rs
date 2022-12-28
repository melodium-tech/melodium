use super::Parameter;
use core::fmt::Debug;
use std::collections::HashMap;

pub trait Parameterized: Debug + Send + Sync {
    fn parameters(&self) -> &HashMap<String, Parameter>;
}

pub trait OrderedParameterized: Debug + Send + Sync {
    fn parameters(&self) -> &Vec<Parameter>;
}
