
use std::fmt::Debug;
use std::collections::HashMap;
use std::sync::Arc;
use super::world::World;
use super::model::Model;
use super::data::Data;
use super::context::Context;

#[derive(Debug)]
struct Environment {
    world: Arc<World>,
    models: HashMap<String, Arc<dyn Model>>,
    variables: HashMap<String, Data>,
    contextes: HashMap<String, Context>,
}

impl Environment {

    fn base(&self) -> Environment {
        Self {
            world: Arc::clone(&self.world),
            models: HashMap::new(),
            variables: HashMap::new(),
            contextes: HashMap::new(),
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

    fn add_variable(&mut self, name: &str, data: Data) {
        self.variables.insert(name.to_string(), data);
    }

    fn get_variable(&self, name: &str) -> Option<&Data> {
        self.variables.get(name)
    }

    fn add_context(&mut self, name: &str, context: Context) {
        self.contextes.insert(name.to_string(), context);
    }

    fn get_context(&self, name: &str) -> Option<&Context> {
        self.contextes.get(name)
    }
}

pub trait GenesisEnvironment : Debug {

    fn base(&self) -> Box<dyn GenesisEnvironment>;
    fn register_model(&self, model: Arc<dyn Model>);
    fn add_model(&mut self, name: &str, model: Arc<dyn Model>);
    fn get_model(&self, name: &str) -> Option<&Arc<dyn Model>>;
    fn add_variable(&mut self, name: &str, data: Data);
    fn get_variable(&self, name: &str) -> Option<&Data>;
}

pub trait ContextualEnvironment : Debug {

    fn base(&self) -> Box<dyn ContextualEnvironment>;
    fn add_model(&mut self, name: &str, model: Arc<dyn Model>);
    fn get_model(&self, name: &str) -> Option<&Arc<dyn Model>>;
    fn add_variable(&mut self, name: &str, data: Data);
    fn get_variable(&self, name: &str) -> Option<&Data>;
    fn add_context(&mut self, name: &str, context: Context);
    fn get_context(&self, name: &str) -> Option<&Context>;
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

    fn add_variable(&mut self, name: &str, data: Data) {
        self.add_variable(name, data)
    }

    fn get_variable(&self, name: &str) -> Option<&Data> {
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

    fn add_variable(&mut self, name: &str, data: Data) {
        self.add_variable(name, data)
    }

    fn get_variable(&self, name: &str) -> Option<&Data> {
        self.get_variable(name)
    }

    fn add_context(&mut self, name: &str, context: Context) {
        self.add_context(name, context)
    }

    fn get_context(&self, name: &str) -> Option<&Context> {
        self.get_context(name)
    }
}
