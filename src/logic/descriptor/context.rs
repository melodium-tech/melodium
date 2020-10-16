
use std::collections::HashMap;
use std::iter::FromIterator;
use super::datatype::DataType;

pub struct Context {
    name: String,
    values: HashMap<String, DataType>,
}

impl Context {
    pub fn new(name: &str, values: Vec<(&str, DataType)>) -> Self {
        Self {
            name: name.to_string(),
            values: HashMap::from_iter(values.iter().map(|v| (v.0.to_string(), v.1)))
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn values(&self) -> &HashMap<String, DataType> {
        &self.values
    }
}
