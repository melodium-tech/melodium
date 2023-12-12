use melodium_common::descriptor::Identifier;
use melodium_engine::designer::Value;
use std::collections::BTreeMap;

pub fn value(value: &Value, names: &BTreeMap<Identifier, String>) -> String {
    match value {
        Value::Raw(val) => val.to_string(),
        Value::Variable(var) => var.clone(),
        Value::Context(context, entry) => {
            format!(
                "{name}[{entry}]",
                name = names.get(context.identifier()).unwrap()
            )
        }
        Value::Function(function, _generics, params) => {
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
