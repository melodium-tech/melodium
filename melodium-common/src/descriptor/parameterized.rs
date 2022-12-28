
use core::fmt::Debug;
use std::collections::HashMap;
use super::Parameter;

pub trait Parameterized : Debug + Send + Sync {
    fn parameters(&self) -> &HashMap<String, Parameter>;
}

pub trait OrderedParameterized : Debug + Send + Sync {
    fn parameters(&self) -> &Vec<Parameter>;
}
