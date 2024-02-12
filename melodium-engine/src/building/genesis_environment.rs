use melodium_common::descriptor::DataType;
use melodium_common::executive::{Model, Value};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct GenesisEnvironment {
    models: HashMap<String, Arc<dyn Model>>,
    generics: HashMap<String, DataType>,
    // All variables that can be determined at static build (all consts and some vars).
    variables: HashMap<String, Value>,
}

impl GenesisEnvironment {
    pub fn new() -> Self {
        Self {
            models: HashMap::new(),
            generics: HashMap::new(),
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

    pub fn add_generic(&mut self, name: &str, generic: DataType) {
        self.generics.insert(name.to_string(), generic);
    }

    pub fn get_generic(&self, name: &str) -> Option<&DataType> {
        self.generics.get(name)
    }

    pub fn generics(&self) -> &HashMap<String, DataType> {
        &self.generics
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
