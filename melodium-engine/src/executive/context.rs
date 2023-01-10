
use std::collections::HashMap;
use std::sync::Arc;
use melodium_common::descriptor::Context as ContextDescriptor;
use melodium_common::executive::{Value, Context as ExecutiveContext};

#[derive(Debug, Clone)]
pub struct Context {
    descriptor: Arc<ContextDescriptor>,
    values: HashMap<String, Value>,
}

impl Context {

    pub fn new(descriptor: Arc<ContextDescriptor>) -> Self {
        Self {
            descriptor,
            values: HashMap::new()
        }
    }

    pub fn set_value(&mut self, name: &str, value: Value) {
        self.values.insert(name.to_string(), value);
    } 
}

impl ExecutiveContext for Context {
    fn descriptor(&self) -> Arc<ContextDescriptor> {
        self.descriptor.clone()
    }

    fn get_value(&self, name: &str) -> &Value {
        self.values.get(name).unwrap()
    }
}
