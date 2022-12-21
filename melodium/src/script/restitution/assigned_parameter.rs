
use crate::logic::designer::{ParameterDesigner};
use super::script::Uses;

pub fn assigned_parameter(uses: &Uses, param: &ParameterDesigner) -> String {

    format!("{} = {}", param.name(), super::value::value(uses, param.value().as_ref().unwrap()))
}
