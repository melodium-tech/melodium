
use std::fmt::Debug;
use std::collections::HashMap;
use super::parameter::Parameter;

pub trait Parameterized : Debug {
    fn parameters(&self) -> &HashMap<String, Parameter>;
}
