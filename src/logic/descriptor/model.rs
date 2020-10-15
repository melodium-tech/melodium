
use std::collections::HashMap;
use super::identified::Identified;
use super::parameter::Parameter;
use super::core_model::CoreModel;

pub trait Model: Identified {
    fn parameters(&self) -> &HashMap<String, Parameter>;
    fn is_core_model(&self) -> bool;
    fn core_model(&self) -> &CoreModel;
}