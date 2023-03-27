
use crate::logic::designer::{ValueDesigner};
use super::script::Uses;

pub fn value(uses: &Uses, value: &ValueDesigner) -> String {

    match value {
        ValueDesigner::Raw(v) => v.to_string(),
        ValueDesigner::Variable(name) => name.clone(),
        ValueDesigner::Context((context, name)) => format!("{}[{}]", context, name),
        ValueDesigner::Function(descriptor, values) => {
            format!("{}({})",
                uses.get(descriptor.identifier()),
                values.iter().map(|v| super::value::value(uses, v)).collect::<Vec<_>>().join(", ")
            )
        }
    }
}
