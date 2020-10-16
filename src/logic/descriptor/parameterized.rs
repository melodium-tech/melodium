
use std::collections::HashMap;
use super::parameter::Parameter;

pub trait Parameterized {
    fn parameters(&self) -> &HashMap<String, Parameter>;
}
