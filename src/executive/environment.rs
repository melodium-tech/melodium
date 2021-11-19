
use std::hash::Hash;
use std::fmt::Debug;
use std::collections::HashMap;
use std::sync::Arc;
use super::world::World;
use super::model::{Model, ModelId};
use super::value::Value;
use super::context::Context;
use super::transmitter::Transmitter;

#[derive(Debug, Clone)]
pub struct GenesisEnvironment {
    world: Arc<World>,
    models: HashMap<String, Arc<dyn Model>>,
    variables: HashMap<String, Value>,
}

impl GenesisEnvironment {

    pub fn new(world: Arc<World>) -> Self {
        Self {
            world,
            models: HashMap::new(),
            variables: HashMap::new(),
        }
    }

    pub fn base(&self) -> Self {
        Self {
            world: Arc::clone(&self.world),
            models: HashMap::new(),
            variables: HashMap::new(),
        }
    }

    pub fn register_model(&self, model: Arc<dyn Model>) -> ModelId  {

        self.world.add_model(Arc::clone(&model))
    }

    pub fn world(&self) -> Arc<World> {

        Arc::clone(&self.world)
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

#[derive(Debug, Clone)]
pub struct ContextualEnvironment {
    world: Arc<World>,
    track_id: u64,
    models: HashMap<String, Arc<dyn Model>>,
    variables: HashMap<String, Value>,
    contexts: HashMap<String, Context>,
    inputs: HashMap<String, Transmitter>,
}

impl ContextualEnvironment {

    pub fn new(world: Arc<World>, track_id: u64) -> Self {
        Self {
            world,
            track_id,
            models: HashMap::new(),
            variables: HashMap::new(),
            contexts: HashMap::new(),
            inputs: HashMap::new(),
        }
    }

    pub fn base(&self) -> Self {
        Self {
            world: Arc::clone(&self.world),
            track_id: self.track_id,
            models: HashMap::new(),
            variables: HashMap::new(),
            contexts: HashMap::new(),
            inputs: HashMap::new(),
        }
    }

    pub fn track_id(&self) -> u64 {
        self.track_id
    }

    pub fn add_model(&mut self, name: &str, model: Arc<dyn Model>) {
        self.models.insert(name.to_string(), model);
    }

    pub fn get_model(&self, name: &str) -> Option<&Arc<dyn Model>> {
        self.models.get(name)
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

    pub fn add_context(&mut self, name: &str, context: Context) {
        self.contexts.insert(name.to_string(), context);
    }

    pub fn get_context(&self, name: &str) -> Option<&Context> {
        self.contexts.get(name)
    }

    pub fn add_input(&mut self, name: &str, input: Transmitter) {
        self.inputs.insert(name.to_string(), input);
    }

    pub fn get_input(&self, name: &str) -> Option<&Transmitter> {
        self.inputs.get(name)
    }
}
