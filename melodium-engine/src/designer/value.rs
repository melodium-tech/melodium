use core::fmt::{Display, Formatter, Result};
use melodium_common::descriptor::{Context, DescribedType, Function, Identifier};
use melodium_common::executive::Value as ExecutiveValue;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub enum Value {
    Raw(ExecutiveValue),
    Variable(String),
    Context(Arc<dyn Context>, String),
    Function(
        Arc<dyn Function>,
        HashMap<String, DescribedType>,
        Vec<Value>,
    ),
}

impl Value {
    pub fn make_use(&self, identifier: &Identifier) -> bool {
        match self {
            Value::Raw(_) => false,
            Value::Variable(_) => false,
            Value::Context(context, _) => context.identifier() == identifier,
            Value::Function(function, _described_types, values) => {
                function.identifier() == identifier
                    || values.iter().any(|value| value.make_use(identifier))
            }
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Value::Raw(data) => write!(f, "{}", data),
            Value::Variable(name) => write!(f, "{}", name),
            Value::Context(desc, entry) => write!(f, "{}[{}]", desc.name(), entry),
            Value::Function(desc, described_types, params) => write!(
                f,
                "{}{}({})",
                desc.identifier().name(),
                if desc.generics().is_empty() {
                    "".to_string()
                } else {
                    format!(
                        "<{}>",
                        desc.generics()
                            .iter()
                            .map(|gen| if let Some(val) = described_types.get(&gen.name) {
                                val.to_string()
                            } else {
                                "_".to_string()
                            })
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                },
                params
                    .iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }
}
