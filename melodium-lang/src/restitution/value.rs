use melodium_common::descriptor::Identifier;
use melodium_engine::designer::Value;
use std::collections::HashMap;

pub fn value(value: &Value, names: &HashMap<Identifier, String>) -> String {
    match value {
        Value::Raw(val) => val.to_string(),
        Value::Variable(var) => var.clone(),
        Value::Context(context, entry) => {
            format!(
                "{name}[{entry}]",
                name = names.get(context.identifier()).unwrap()
            )
        }
        Value::Function(function, params) => {
            let name = names.get(function.identifier()).unwrap();
            let params = params
                .iter()
                .map(|p| self::value(p, names))
                .collect::<Vec<_>>()
                .join(", ");

            format!("{name}({params})")
        }
    }
}
