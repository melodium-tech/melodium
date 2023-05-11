use core::fmt::{Display, Formatter, Result};
use melodium_common::descriptor::{Context, Function};
use melodium_common::executive::Value as ExecutiveValue;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub enum Value {
    Raw(ExecutiveValue),
    Variable(String),
    Context(Arc<dyn Context>, String),
    Function(Arc<dyn Function>, Vec<Value>),
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Value::Raw(data) => write!(f, "{}", data),
            Value::Variable(name) => write!(f, "{}", name),
            Value::Context(id, entry) => write!(f, "{}[{}]", id.name(), entry),
            Value::Function(id, params) => write!(
                f,
                "{}({})",
                id.identifier().name(),
                params
                    .iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }
}
