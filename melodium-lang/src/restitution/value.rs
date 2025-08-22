use crate::restitution::describe_type;
use melodium_common::descriptor::Identifier;
use melodium_engine::designer::Value;
use std::collections::BTreeMap;

pub fn value(value: &Value, names: &BTreeMap<Identifier, String>, level: usize) -> String {
    match value {
        Value::Raw(val) => val.to_string(),
        Value::Array(array) => format!(
            "[{}]",
            array
                .iter()
                .map(|val| self::value(val, names, level + 1))
                .collect::<Vec<_>>()
                .join(&format!(",\n{}", "    ".repeat(level)))
        ),
        Value::Variable(var) => var.clone(),
        Value::Context(context, entry) => {
            format!(
                "{name}[{entry}]",
                name = names.get(context.identifier()).unwrap()
            )
        }
        Value::Function(function, generics, params) => {
            let name = names.get(function.identifier()).unwrap();

            let generics = if !function.generics().is_empty() && !generics.is_empty() {
                format!(
                    "<{}>",
                    function
                        .generics()
                        .iter()
                        .map(|generic| generics
                            .get(&generic.name)
                            .map(|desc_type| describe_type(desc_type, names))
                            .unwrap_or_else(|| "_".to_string()))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            } else {
                String::new()
            };

            let params = params
                .iter()
                .map(|p| self::value(p, names, level + 1))
                .collect::<Vec<_>>()
                .join(&format!(",\n{}", "    ".repeat(level)));

            format!(
                "{name}{generics}(\n{lvl}{params}\n{lvl})",
                lvl = "    ".repeat(level)
            )
        }
    }
}
