use crate::restitution::describe_type;
use melodium_common::descriptor::Identifier;
use melodium_common::executive::Value as ExecutiveValue;
use melodium_engine::designer::Value;
use std::collections::BTreeMap;

/// Gives the literal of a raw value, secrets being written as their locator.
///
/// Secrets without locator have no literal, they keep their placeholder.
fn raw(value: &ExecutiveValue) -> String {
    match value {
        ExecutiveValue::Secret(secret) => match secret.locator() {
            Some(locator) => ExecutiveValue::String(locator.to_string()).to_string(),
            None => secret.to_string(),
        },
        ExecutiveValue::Vec(values) => format!(
            "[{}]",
            values.iter().map(raw).collect::<Vec<_>>().join(", ")
        ),
        ExecutiveValue::Option(Some(value)) => raw(value),
        other => other.to_string(),
    }
}

pub fn value(value: &Value, names: &BTreeMap<Identifier, String>, level: usize) -> String {
    match value {
        Value::Raw(val) => raw(val),
        Value::Array(array) => format!(
            "[{opt_n}{opt_space}{array}{opt_n}{opt_space_n1}]",
            opt_n = if array.is_empty() { "" } else { "\n" },
            opt_space = if array.is_empty() {
                "".to_string()
            } else {
                "    ".repeat(level)
            },
            opt_space_n1 = if array.is_empty() {
                "".to_string()
            } else {
                "    ".repeat(level.saturating_sub(1))
            },
            array = array
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
                "{name}{generics}({opt_n}{opt_space}{params}{opt_n}{opt_space_n1})",
                opt_n = if params.is_empty() { "" } else { "\n" },
                opt_space = if params.is_empty() {
                    "".to_string()
                } else {
                    "    ".repeat(level)
                },
                opt_space_n1 = if params.is_empty() {
                    "".to_string()
                } else {
                    "    ".repeat(level.saturating_sub(1))
                }
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use melodium_common::descriptor::DataType;
    use melodium_common::executive::{Secret, SecretOrigin, SecretPolicy};

    #[test]
    fn secrets_are_written_as_their_locator() {
        let secret = |locator: &str| {
            ExecutiveValue::Secret(Secret::from_locator(locator, DataType::String).unwrap())
        };
        let names = BTreeMap::new();

        assert_eq!(
            value(&Value::Raw(secret("env:DB_PASSWORD")), &names, 1),
            "\"env:DB_PASSWORD\""
        );
        assert_eq!(
            value(
                &Value::Raw(ExecutiveValue::Vec(vec![
                    secret("file:/run/secrets/a"),
                    secret("vault:kv/data/b#c")
                ])),
                &names,
                1
            ),
            "[\"file:/run/secrets/a\", \"vault:kv/data/b#c\"]"
        );
        assert_eq!(
            value(
                &Value::Raw(ExecutiveValue::Option(Some(Box::new(secret("env:A"))))),
                &names,
                1
            ),
            "\"env:A\""
        );

        let inline = Secret::new(
            "runtime".to_string(),
            DataType::String,
            SecretPolicy::default(),
            SecretOrigin::Inline(ExecutiveValue::String("hidden-value".to_string())),
        )
        .unwrap();
        assert_eq!(
            value(&Value::Raw(ExecutiveValue::Secret(inline)), &names, 1),
            "<secret \"runtime\">"
        );
    }
}
