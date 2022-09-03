
use crate::logic::descriptor::{ParameterDescriptor, VariabilityDescriptor};

pub fn declared_parameter(param: &ParameterDescriptor) -> String {

    let mut result = String::new();

    result.push_str(match param.variability() {
        VariabilityDescriptor::Const => "const ",
        VariabilityDescriptor::Var => "var ",
    });

    result.push_str(param.name());

    result.push_str(": ");
    result.push_str(&param.datatype().to_string());

    if let Some(default) = param.default() {
        result.push_str(&format!(" = {}", default));
    }

    result
}
