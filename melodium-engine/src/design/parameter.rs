use super::Value;

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub value: Value,
}
