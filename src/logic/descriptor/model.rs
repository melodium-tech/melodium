
use std::collections::HashMap;
use super::identified::Identified;
use super::parameter::Parameter;

pub trait Model: Identified {
    fn parameters(&self) -> &HashMap<String, Parameter>;
}