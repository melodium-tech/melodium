
use std::fmt::Debug;
use std::collections::HashMap;
use std::sync::Arc;
use super::world::World;
use super::model::Model;
use super::value::Value;
use super::context::Context;
use super::transmitter::Transmitter;

#[derive(Debug)]
pub struct Environment {
    world: Arc<World>,
    models: HashMap<String, Arc<dyn Model>>,
    variables: HashMap<String, Value>,
    contexts: HashMap<String, Context>,
    inputs: HashMap<String, Transmitter>,
}

impl Environment {

    pub fn new(world: Arc<World>) -> Self {
        Self {
            world,
            models: HashMap::new(),
            variables: HashMap::new(),
            contexts: HashMap::new(),
            inputs: HashMap::new(),
        }
    }

    fn base(&self) -> Environment {
        Self {
            world: Arc::clone(&self.world),
            models: HashMap::new(),
            variables: HashMap::new(),
            contexts: HashMap::new(),
            inputs: HashMap::new(),
        }
    }

    fn register_model(&self, model: Arc<dyn Model>) {

        self.world.add_model(Arc::clone(&model));
    }

    fn add_model(&mut self, name: &str, model: Arc<dyn Model>) {
        self.models.insert(name.to_string(), model);
    }

    fn get_model(&self, name: &str) -> Option<&Arc<dyn Model>> {
        self.models.get(name)
    }

    fn add_variable(&mut self, name: &str, value: Value) {
        self.variables.insert(name.to_string(), value);
    }

    fn get_variable(&self, name: &str) -> Option<&Value> {
        self.variables.get(name)
    }

    fn add_context(&mut self, name: &str, context: Context) {
        self.contexts.insert(name.to_string(), context);
    }

    fn get_context(&self, name: &str) -> Option<&Context> {
        self.contexts.get(name)
    }

    fn add_input(&mut self, name: &str, input: Transmitter) {
        self.inputs.insert(name.to_string(), input);
    }

    fn get_input(&self, name: &str) -> Option<&Transmitter> {
        self.inputs.get(name)
    }
}

pub trait GenesisEnvironment : Debug {

    fn base(&self) -> Box<dyn GenesisEnvironment>;
    fn register_model(&self, model: Arc<dyn Model>);
    fn add_model(&mut self, name: &str, model: Arc<dyn Model>);
    fn get_model(&self, name: &str) -> Option<&Arc<dyn Model>>;
    fn add_variable(&mut self, name: &str, value: Value);
    fn get_variable(&self, name: &str) -> Option<&Value>;
}

pub trait ContextualEnvironment : Debug {

    fn base(&self) -> Box<dyn ContextualEnvironment>;
    fn add_model(&mut self, name: &str, model: Arc<dyn Model>);
    fn get_model(&self, name: &str) -> Option<&Arc<dyn Model>>;
    fn add_variable(&mut self, name: &str, value: Value);
    fn get_variable(&self, name: &str) -> Option<&Value>;
    fn add_context(&mut self, name: &str, context: Context);
    fn get_context(&self, name: &str) -> Option<&Context>;
    fn add_input(&mut self, name: &str, input: Transmitter);
    fn get_input(&self, name: &str) -> Option<&Transmitter>;
}

impl GenesisEnvironment for Environment {

    fn base(&self) -> Box<dyn GenesisEnvironment> {
        Box::new(self.base())
    }

    fn register_model(&self, model: Arc<dyn Model>) {
        self.register_model(model)
    }

    fn add_model(&mut self, name: &str, model: Arc<dyn Model>) {
        self.add_model(name, model)
    }

    fn get_model(&self, name: &str) -> Option<&Arc<dyn Model>> {
        self.get_model(name)
    }

    fn add_variable(&mut self, name: &str, value: Value) {
        self.add_variable(name, value)
    }

    fn get_variable(&self, name: &str) -> Option<&Value> {
        self.get_variable(name)
    }
}

impl ContextualEnvironment for Environment {

    fn base(&self) -> Box<dyn ContextualEnvironment> {
        Box::new(self.base())
    }

    fn add_model(&mut self, name: &str, model: Arc<dyn Model>) {
        self.add_model(name, model)
    }

    fn get_model(&self, name: &str) -> Option<&Arc<dyn Model>> {
        self.get_model(name)
    }

    fn add_variable(&mut self, name: &str, value: Value) {
        self.add_variable(name, value)
    }

    fn get_variable(&self, name: &str) -> Option<&Value> {
        self.get_variable(name)
    }

    fn add_context(&mut self, name: &str, context: Context) {
        self.add_context(name, context)
    }

    fn get_context(&self, name: &str) -> Option<&Context> {
        self.get_context(name)
    }

    fn add_input(&mut self, name: &str, input: Transmitter) {
        self.add_input(name, input);
    }

    fn get_input(&self, name: &str) -> Option<&Transmitter> {
        self.get_input(name)
    }
}
