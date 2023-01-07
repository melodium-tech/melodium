
use std::collections::HashMap;
use std::sync::Arc;
use melodium_common::executive::{Model, ModelId, Value};
use crate::world::World;

#[derive(Debug, Clone)]
pub struct GenesisEnvironment {
    models: HashMap<String, Arc<dyn Model>>,
    variables: HashMap<String, Value>,
}

impl GenesisEnvironment {

    pub fn new() -> Self {
        Self {
            models: HashMap::new(),
            variables: HashMap::new(),
        }
    }

    pub fn base(&self) -> Self {
        Self {
            models: HashMap::new(),
            variables: HashMap::new(),
        }
    }

    pub fn add_model(&mut self, name: &str, model: Arc<dyn Model>) {
        self.models.insert(name.to_string(), model);
    }

    pub fn get_model(&self, name: &str) -> Option<&Arc<dyn Model>> {
        self.models.get(name)
    }

    pub fn models(&self) -> &HashMap<String, Arc<dyn Model>> {
        &self.models
    }

    pub fn add_variable(&mut self, name: &str, value: Value) {
        self.variables.insert(name.to_string(), value);
    }

    pub fn get_variable(&self, name: &str) -> Option<&Value> {
        self.variables.get(name)
    }

    pub fn variables(&self) -> &HashMap<String, Value> {
        &self.variables
    }
}
