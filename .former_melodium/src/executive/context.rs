
use std::collections::HashMap;
use super::value::Value;

#[derive(Debug, Clone)]
pub struct Context {
    values: HashMap<String, Value>,
}

impl Context {

    pub fn new() -> Self {
        Self {
            values: HashMap::new()
        }
    }

    pub fn get_value(&self, name: &str) -> Option<&Value> {
        self.values.get(name)
    }

    pub fn set_value(&mut self, name: &str, value: Value) {
        self.values.insert(name.to_string(), value);
    } 
}
